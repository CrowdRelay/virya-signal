package music.virya.signal.push

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.os.Build
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

@TauriPlugin(
    permissions = [
        Permission(strings = [Manifest.permission.POST_NOTIFICATIONS], alias = ALIAS_NOTIFICATION)
    ]
)
class SignalPushPlugin(private val activity: Activity) : Plugin(activity) {
    private var pendingLaunchTarget: String? = null

    override fun onNewIntent(intent: Intent) {
        launchTargetFrom(intent)?.let { pendingLaunchTarget = it }
    }
    @Command
    fun getToken(invoke: Invoke) {
        if (FirebaseApp.getApps(activity.applicationContext).isEmpty()) {
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
