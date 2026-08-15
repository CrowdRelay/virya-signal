package music.virya.signal.push

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import java.net.HttpURLConnection
import java.net.URL
import java.util.UUID
import kotlin.concurrent.thread

private const val CHANNEL_ID = "virya_signal_push"
private const val ACK_BASE = "https://signal-api.virya.music/v1/public/push/deliveries/"
private val ACK_TOKEN = Regex("^[A-Za-z0-9_-]{32,200}$")

class ViryaFirebaseMessagingService : FirebaseMessagingService() {
    override fun onMessageReceived(message: RemoteMessage) {
        val data = message.data
        val deliveryId = data["delivery_id"]?.trim().orEmpty()
        val ackToken = data["ack_token"]?.trim().orEmpty()
        val title = data["title"]?.trim().orEmpty()
        val body = data["body"]?.trim().orEmpty()
        val targetPath = data["target_path"]?.trim().orEmpty()
        val collapseKey = data["collapse_key"]?.trim().orEmpty()
        if (!validPayload(deliveryId, ackToken, title, body, targetPath, collapseKey)) return

        val manager = NotificationManagerCompat.from(this)
        if (!manager.areNotificationsEnabled()) return
        ensureChannel()

        val launchIntent = packageManager.getLaunchIntentForPackage(packageName) ?: return
        launchIntent.action = Intent.ACTION_VIEW
        launchIntent.putExtra("virya_push_target_path", targetPath)
        launchIntent.addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP)
        val pending = PendingIntent.getActivity(
            this,
            deliveryId.hashCode(),
            launchIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        val notificationIcon = resources
            .getIdentifier("virya_signal_notification", "drawable", packageName)
            .takeIf { it != 0 }
            ?: applicationInfo.icon.takeIf { it != 0 }
            ?: android.R.drawable.ic_dialog_info
        val notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(notificationIcon)
            .setContentTitle(title)
            .setContentText(body)
            .setStyle(NotificationCompat.BigTextStyle().bigText(body))
            .setAutoCancel(true)
            .setContentIntent(pending)
            .setCategory(NotificationCompat.CATEGORY_EVENT)
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .build()
        manager.notify(collapseKey.ifEmpty { deliveryId }, deliveryId.hashCode(), notification)

        // The acknowledgement is deliberately sent only after NotificationManager
        // accepted the local display operation. Retries repeat only the ACK, never
        // the notification display itself.
        acknowledgeAsync(deliveryId, ackToken)
    }

    override fun onNewToken(token: String) {
        // The fan session never enters Android plaintext storage. Rotation is
        // reconciled on the next app unlock/resume when Rust asks Firebase for
        // the current token and registers it with CrowdRelay using Stronghold.
        getSharedPreferences("virya_signal_push", Context.MODE_PRIVATE)
            .edit()
            .putLong("token_rotated_at_ms", System.currentTimeMillis())
            .apply()
    }

    private fun ensureChannel() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val service = getSystemService(NotificationManager::class.java)
        if (service.getNotificationChannel(CHANNEL_ID) != null) return
        service.createNotificationChannel(
            NotificationChannel(CHANNEL_ID, "VIRYA Signal", NotificationManager.IMPORTANCE_DEFAULT).apply {
                description = "Koncerty, bilety i ważne wiadomości VIRYA Signal"
            }
        )
    }

    private fun acknowledgeAsync(deliveryId: String, ackToken: String) {
        thread(name = "virya-push-ack", isDaemon = true) {
            val waits = longArrayOf(0L, 500L, 2_000L)
            for (wait in waits) {
                if (wait > 0) Thread.sleep(wait)
                if (acknowledgeOnce(deliveryId, ackToken)) return@thread
            }
        }
    }

    private fun acknowledgeOnce(deliveryId: String, ackToken: String): Boolean {
        val connection = try {
            (URL("$ACK_BASE$deliveryId/ack").openConnection() as HttpURLConnection).apply {
                requestMethod = "POST"
                connectTimeout = 5_000
                readTimeout = 5_000
                instanceFollowRedirects = false
                doOutput = true
                setRequestProperty("Content-Type", "application/json")
                setRequestProperty("Cache-Control", "no-store")
            }
        } catch (_: Exception) {
            return false
        }
        return try {
            val escaped = ackToken.replace("\\", "\\\\").replace("\"", "\\\"")
            connection.outputStream.use { stream ->
                stream.write("{\"ack_token\":\"$escaped\"}".toByteArray(Charsets.UTF_8))
            }
            val status = connection.responseCode
            status in 200..299 || (status in 400..499 && status != 408 && status != 429)
        } catch (_: Exception) {
            false
        } finally {
            connection.disconnect()
        }
    }

    private fun validPayload(
        deliveryId: String,
        ackToken: String,
        title: String,
        body: String,
        targetPath: String,
        collapseKey: String,
    ): Boolean {
        if (runCatching { UUID.fromString(deliveryId) }.isFailure) return false
        if (!ACK_TOKEN.matches(ackToken)) return false
        if (title.isEmpty() || title.length > 160) return false
        if (body.isEmpty() || body.length > 1_200) return false
        if (!targetPath.startsWith("/") || targetPath.startsWith("//") || targetPath.length > 512) return false
        if (collapseKey.length > 160) return false
        return true
    }
}
