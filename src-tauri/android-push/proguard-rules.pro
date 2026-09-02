# Virya Signal — R8/ProGuard keep rules for release builds.
#
# Tauri generates proguard-wry.pro (keeps WryActivity, Ipc, RustWebView, etc.)
# and proguard-tauri.pro (keeps TauriActivity.getPluginManager). Both are
# included in proguardFiles by prepare-android.py. This file covers everything
# the generated rules do not: the Tauri plugin runtime, Firebase Messaging,
# our push plugin, and WebView JS interfaces.

# ── Keep line numbers for crash reports ──
-keepattributes SourceFile,LineNumberTable
-renamesourcefileattribute SourceFile

# ── Tauri plugin runtime ──
# Tauri plugins are discovered via reflection at runtime. The annotation
# processors generate registration code that R8 can strip without these keeps.
-keep class app.tauri.** { *; }
-keep @app.tauri.annotation.TauriPlugin class * { *; }
-keep class * extends app.tauri.plugin.Plugin { *; }
-keepclasseswithmembernames class * {
    @app.tauri.annotation.Command <methods>;
}
-keepclasseswithmembernames class * {
    @app.tauri.annotation.Permission <methods>;
}

# ── Virya Signal app classes ──
# The native Rust code calls methods (getId, getVersion, setWebView, etc.)
# via JNI on the MainActivity instance. MainActivity extends TauriActivity
# extends WryActivity. proguard-wry.pro keeps WryActivity and proguard-tauri.pro
# keeps TauriActivity.getPluginManager, but R8 can still rename inherited
# methods on subclasses, breaking JNI lookups. Keep the entire app package
# — it is ~11 Kotlin files; R8 still obfuscates the dependencies (Tauri,
# Firebase, AndroidX) which are the bulk of the code and satisfy the Play
# obfuscation threshold.
-keep class music.virya.signal.** { *; }

# ── Virya Signal push plugin ──
-keep class music.virya.signal.push.** { *; }

# ── Firebase Messaging ──
# Firebase uses reflection for service discovery and model deserialization.
-keep class com.google.firebase.** { *; }
-keep class com.google.android.gms.** { *; }
-dontwarn com.google.firebase.**
-dontwarn com.google.android.gms.**

# Firebase MessagingService subclass is instantiated by reflection.
-keep class * extends com.google.firebase.messaging.FirebaseMessagingService {
    public <init>(...);
    public void onMessageReceived(...);
    public void onNewToken(...);
}

# ── WebView / JavaScript interface ──
# The IPC bridge is already covered by proguard-wry.pro, but keep any
# additional JS interfaces that might be added.
-keepclassmembers class * {
    @android.webkit.JavascriptInterface <methods>;
}

# ── AndroidX / Material ──
# These are generally safe to shrink, but some lifecycle and activity
# components use reflection. Keep the entry points.
-keep class androidx.activity.** { *; }
-keep class androidx.lifecycle.** { *; }
-keep class androidx.appcompat.** { *; }
-keep class androidx.webkit.** { *; }

# ── Kotlin metadata ──
# Keep Kotlin metadata for reflection-based libraries.
-keepattributes RuntimeVisibleAnnotations,RuntimeVisibleParameterAnnotations,RuntimeVisibleTypeAnnotations
-keepattributes AnnotationDefault
-keep class kotlin.Metadata { *; }

# ── Native methods ──
# Already covered by proguard-wry.pro, but be explicit.
-keepclasseswithmembernames class * {
    native <methods>;
}

# ── Enum classes ──
# R8 can break enum valueOf / values if the class is renamed.
-keepclassmembers enum * {
    public static **[] values();
    public static ** valueOf(java.lang.String);
}
