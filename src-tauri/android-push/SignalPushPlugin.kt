package music.virya.signal.push

import android.Manifest
import android.app.Activity
import android.os.Build
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

@TauriPlugin(
    permissions = [
        Permission(strings = [Manifest.permission.POST_NOTIFICATIONS], alias = ALIAS_NOTIFICATION)
    ]
)
class SignalPushPlugin(private val activity: Activity) : Plugin(activity) {
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
    fun deleteToken(invoke: Invoke) {
        if (FirebaseApp.getApps(activity.applicationContext).isEmpty()) {
            invoke.resolve()
            return
        }
        FirebaseMessaging.getInstance().deleteToken().addOnCompleteListener { task ->
            if (task.isSuccessful) invoke.resolve() else invoke.reject("fcm_token_delete_failed")
        }
    }

    @Command
    override fun checkPermissions(invoke: Invoke) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            permissionState(invoke)
        } else {
            super.checkPermissions(invoke)
        }
    }

    @Command
    override fun requestPermissions(invoke: Invoke) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            permissionState(invoke)
            return
        }
        if (getPermissionState(ALIAS_NOTIFICATION) == PermissionState.GRANTED) {
            permissionState(invoke)
        } else {
            requestPermissionForAlias(ALIAS_NOTIFICATION, invoke, "permissionCallback")
        }
    }

    @PermissionCallback
    private fun permissionCallback(invoke: Invoke) {
        permissionState(invoke)
    }

    private fun permissionState(invoke: Invoke) {
        val result = JSObject()
        result.put(
            "permissionState",
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
                "granted"
            } else {
                getPermissionState(ALIAS_NOTIFICATION).toString().lowercase()
            }
        )
        invoke.resolve(result)
    }
}
