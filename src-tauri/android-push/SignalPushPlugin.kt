package music.virya.signal.push

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.os.Build
import android.net.Uri
import android.provider.Settings
import androidx.core.app.NotificationManagerCompat
import app.tauri.PermissionState
import app.tauri.annotation.Command
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import com.google.firebase.FirebaseApp
import com.google.firebase.messaging.FirebaseMessaging

private const val ALIAS_NOTIFICATION = "notification"
private const val EXTRA_PUSH_TARGET_PATH = "virya_push_target_path"
private const val MAX_APP_LINK_BYTES = 1024

@TauriPlugin(
    permissions = [
        Permission(strings = [Manifest.permission.POST_NOTIFICATIONS], alias = ALIAS_NOTIFICATION)
    ]
)
class SignalPushPlugin(private val activity: Activity) : Plugin(activity) {
    private var pendingLaunchTarget: String? = null
    private var pendingAppLink: String? = null
    private var pendingAppLinkRejected = false
    private var pendingSynesthesiaAppLink: String? = null
    private var pendingSynesthesiaAppLinkRejected = false
    private var pendingFanConfirmAppLink: String? = null
    private var pendingFanConfirmAppLinkRejected = false

    override fun onNewIntent(intent: Intent) {
        launchTargetFrom(intent)?.let { pendingLaunchTarget = it }
        // An intent aimed at a Latarnik path is always an answer, even when the
        // capability inside it is malformed. Record the refusal so the shell can
        // say so instead of leaving the user on an unchanged screen.
        if (isLatarnikIntent(intent)) {
            val link = appLinkFrom(intent)
            pendingAppLink = link
            pendingAppLinkRejected = link == null
        }
        if (isSynesthesiaIntent(intent)) {
            val link = synesthesiaAppLinkFrom(intent)
            pendingSynesthesiaAppLink = link
            pendingSynesthesiaAppLinkRejected = link == null
        }
        if (isFanConfirmIntent(intent)) {
            val link = fanConfirmAppLinkFrom(intent)
            pendingFanConfirmAppLink = link
            pendingFanConfirmAppLinkRejected = link == null
        }
    }
    private fun ensureFirebaseInitialized(): Boolean {
        val context = activity.applicationContext
        if (FirebaseApp.getApps(context).isNotEmpty()) {
            return true
        }
        // FirebaseInitProvider normally creates the default app before the
        // Activity starts. A Play/WebView lifecycle edge must not make push
        // depend on that ordering: initialize explicitly from the compiled
        // google-services resources and still fail closed when they are absent.
        return FirebaseApp.initializeApp(context) != null
    }

    @Command
    fun getFirebaseState(invoke: Invoke) {
        val result = JSObject()
        result.put("configured", ensureFirebaseInitialized())
        invoke.resolve(result)
    }

    @Command
    fun getToken(invoke: Invoke) {
        if (!ensureFirebaseInitialized()) {
            invoke.reject("firebase_not_configured")
            return
        }
        FirebaseMessaging.getInstance().token.addOnCompleteListener { task ->
            if (!task.isSuccessful) {
                invoke.reject("fcm_token_unavailable")
                return@addOnCompleteListener
            }
            val token = task.result?.trim().orEmpty()
            if (token.length < 16 || token.length > 4096) {
                invoke.reject("fcm_token_invalid")
                return@addOnCompleteListener
            }
            val result = JSObject()
            result.put("token", token)
            invoke.resolve(result)
        }
    }

    @Command
    fun getNotificationPermissionState(invoke: Invoke) {
        permissionState(invoke)
    }

    @Command
    fun requestNotificationPermission(invoke: Invoke) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            permissionState(invoke)
            return
        }
        when (getPermissionState(ALIAS_NOTIFICATION)) {
            PermissionState.GRANTED, PermissionState.DENIED -> permissionState(invoke)
            else -> requestPermissionForAlias(ALIAS_NOTIFICATION, invoke, "permissionCallback")
        }
    }

    @Command
    fun takeLaunchTarget(invoke: Invoke) {
        val target = pendingLaunchTarget ?: launchTargetFrom(activity.intent)
        pendingLaunchTarget = null
        activity.intent.removeExtra(EXTRA_PUSH_TARGET_PATH)
        val result = JSObject()
        result.put("targetPath", target.orEmpty())
        invoke.resolve(result)
    }


    @Command
    fun takeAppLink(invoke: Invoke) {
        val currentIsLatarnik = isLatarnikIntent(activity.intent)
        val currentLink = appLinkFrom(activity.intent)
        val link = pendingAppLink ?: currentLink
        val rejected = pendingAppLinkRejected || (currentIsLatarnik && currentLink == null)
        pendingAppLink = null
        pendingAppLinkRejected = false
        // One-time either way: a refused capability must not be re-offered on
        // the next resume any more than an accepted one.
        if (currentIsLatarnik && (currentLink == null || currentLink == link)) {
            activity.intent.data = null
        }
        val result = JSObject()
        result.put("appLink", link.orEmpty())
        result.put("rejected", rejected)
        invoke.resolve(result)
    }

    @Command
    fun takeSynesthesiaAppLink(invoke: Invoke) {
        val currentIsSynesthesia = isSynesthesiaIntent(activity.intent)
        val currentLink = synesthesiaAppLinkFrom(activity.intent)
        val link = pendingSynesthesiaAppLink ?: currentLink
        val rejected =
            pendingSynesthesiaAppLinkRejected || (currentIsSynesthesia && currentLink == null)
        pendingSynesthesiaAppLink = null
        pendingSynesthesiaAppLinkRejected = false
        // Retain accepted data on Activity.intent until Rust acknowledges a
        // terminal claim outcome. Android can then recreate the process and
        // re-offer the short-lived capability after an auth transition.
        if (currentIsSynesthesia && currentLink == null) {
            activity.intent.data = null
        }
        val result = JSObject()
        result.put("appLink", link.orEmpty())
        result.put("rejected", rejected)
        invoke.resolve(result)
    }

    @Command
    fun takeFanConfirmAppLink(invoke: Invoke) {
        val currentIsFanConfirm = isFanConfirmIntent(activity.intent)
        val currentLink = fanConfirmAppLinkFrom(activity.intent)
        val link = pendingFanConfirmAppLink ?: currentLink
        val rejected = pendingFanConfirmAppLinkRejected || (currentIsFanConfirm && currentLink == null)
        pendingFanConfirmAppLink = null
        pendingFanConfirmAppLinkRejected = false
        // One-time either way. The mailed token is spent by the exchange that
        // follows, so re-offering it on the next resume could only fail.
        if (currentIsFanConfirm) {
            activity.intent.data = null
        }
        val result = JSObject()
        result.put("appLink", link.orEmpty())
        result.put("rejected", rejected)
        invoke.resolve(result)
    }

    @Command
    fun clearSynesthesiaAppLink(invoke: Invoke) {
        pendingSynesthesiaAppLink = null
        pendingSynesthesiaAppLinkRejected = false
        if (isSynesthesiaIntent(activity.intent)) {
            activity.intent.data = null
        }
        invoke.resolve()
    }

    @Command
    fun openNotificationSettings(invoke: Invoke) {
        val intent = Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).apply {
            putExtra(Settings.EXTRA_APP_PACKAGE, activity.packageName)
        }
        activity.startActivity(intent)
        invoke.resolve()
    }

    @PermissionCallback
    private fun permissionCallback(invoke: Invoke) {
        permissionState(invoke)
    }

    private fun launchTargetFrom(intent: Intent?): String? {
        val target = intent?.getStringExtra(EXTRA_PUSH_TARGET_PATH)?.trim().orEmpty()
        return target.takeIf {
            it.startsWith("/") && !it.startsWith("//") && it.length <= 512 &&
                !it.any { character -> character.code < 0x20 || character.code == 0x7f }
        }
    }


    // Whether this intent was addressed to Latarnik at all, independent of the
    // capability being well formed. Anything else is not our intent to answer.
    private fun isLatarnikIntent(intent: Intent?): Boolean {
        if (intent?.action != Intent.ACTION_VIEW) return false
        val uri: Uri = intent.data ?: return false
        if (uri.scheme != "https") return false
        if (uri.host != "virya.music" && uri.host != "www.virya.music") return false
        val path = uri.path?.trimEnd('/').orEmpty()
        return path == "/latarnik" || path == "/pl/latarnik"
    }

    private fun isInviteChar(character: Char): Boolean =
        character in 'A'..'Z' || character in 'a'..'z' || character in '0'..'9' ||
            character == '-' || character == '_'

    private fun appLinkFrom(intent: Intent?): String? {
        if (!isLatarnikIntent(intent)) return null
        val uri: Uri = intent?.data ?: return null
        if (uri.fragment != null || uri.userInfo != null) return null
        if (uri.queryParameterNames != setOf("invite")) return null
        val values = uri.getQueryParameters("invite")
        if (values.size != 1) return null
        val invite = values.single().trim()
        // ASCII only, matching the Rust capability grammar exactly. A wider
        // Unicode class here would hand the shell a link it must reject anyway.
        if (invite.length !in 24..128 || !invite.all { isInviteChar(it) }) return null
        val link = uri.toString()
        return link.takeIf { it.length <= MAX_APP_LINK_BYTES && !it.any { character -> character.code < 0x20 || character.code >= 0x7f } }
    }

    private fun isSynesthesiaIntent(intent: Intent?): Boolean {
        if (intent?.action != Intent.ACTION_VIEW) return false
        val uri: Uri = intent.data ?: return false
        if (uri.port != -1 || uri.userInfo != null) return false
        val path = uri.path?.trimEnd('/').orEmpty()
        val verifiedWeb =
            uri.scheme == "https" &&
                (uri.host == "virya.music" || uri.host == "www.virya.music") &&
                (path == "/my-signal" || path == "/pl/my-signal")
        val native =
            uri.scheme == "virya-signal" &&
                uri.host == "my-signal" &&
                path.isEmpty()
        return verifiedWeb || native
    }

    // Whether this intent was addressed to fan confirmation at all, independent
    // of the token being well formed.
    private fun isFanConfirmIntent(intent: Intent?): Boolean {
        if (intent?.action != Intent.ACTION_VIEW) return false
        val uri: Uri = intent.data ?: return false
        if (uri.port != -1 || uri.userInfo != null) return false
        val path = uri.path?.trimEnd('/').orEmpty()
        val verifiedWeb =
            uri.scheme == "https" &&
                (uri.host == "virya.music" || uri.host == "www.virya.music") &&
                (path == "/signal/confirm" || path == "/pl/signal/confirm")
        val native =
            uri.scheme == "virya-signal" && uri.host == "fan" && path == "/confirm"
        return verifiedWeb || native
    }

    // CrowdRelay mails the token either as a query parameter or in the fragment.
    // The fragment form never reaches the site's server, so both are accepted
    // here and normalised to the same 64 hex characters the native side expects.
    private fun fanConfirmAppLinkFrom(intent: Intent?): String? {
        if (!isFanConfirmIntent(intent)) return null
        val uri: Uri = intent?.data ?: return null
        val fromQuery = runCatching { uri.getQueryParameter("token") }.getOrNull()
        val fromFragment = uri.fragment
            ?.split('&')
            ?.firstNotNullOfOrNull { part ->
                part.removePrefix("token=").takeIf { it != part }
            }
        val token = (fromQuery ?: fromFragment)?.trim().orEmpty()
        if (token.length != 64 || !token.all { isHexChar(it) }) return null
        return token.lowercase()
    }

    private fun isHexChar(character: Char): Boolean =
        character in '0'..'9' || character in 'a'..'f' || character in 'A'..'F'

    private fun synesthesiaAppLinkFrom(intent: Intent?): String? {
        if (!isSynesthesiaIntent(intent)) return null
        val uri: Uri = intent?.data ?: return null
        if (uri.userInfo != null) return null
        if (uri.queryParameterNames != setOf("source")) return null
        val sources = uri.getQueryParameters("source")
        if (sources.size != 1 || sources.single() != "synesthesia") return null
        val fragment = uri.fragment ?: return null
        if (!fragment.startsWith("handoff=")) return null
        val handoff = fragment.removePrefix("handoff=")
        if (handoff.length != 64 || !handoff.all { isHexChar(it) }) return null
        val link = uri.toString()
        return link.takeIf {
            it.length <= MAX_APP_LINK_BYTES &&
                !it.any { character -> character.code < 0x20 || character.code >= 0x7f }
        }
    }

    private fun permissionState(invoke: Invoke) {
        val runtimeState = if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            PermissionState.GRANTED
        } else {
            getPermissionState(ALIAS_NOTIFICATION) ?: PermissionState.PROMPT
        }
        val normalized = if (runtimeState == PermissionState.GRANTED &&
            !NotificationManagerCompat.from(activity).areNotificationsEnabled()
        ) {
            "denied"
        } else {
            runtimeState.toString()
        }
        val result = JSObject()
        result.put("permissionState", normalized)
        invoke.resolve(result)
    }
}
