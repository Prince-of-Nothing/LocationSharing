package org.alwaystogether.mobile

import android.util.Log

/**
 * Minimal in-app crash + lifecycle logger (zero dependencies).
 *
 * Writes to logcat with tag `Pinpoint`. No Firebase, no analytics,
 * no network â€” purely local debug output for debugging crashes during
 * development and local testing.
 *
 * Every public method is safe to call from any thread.
 */
object CrashLogger {

    private const val TAG = "Pinpoint"

    /** Log a caught exception with context so it shows up in logcat. */
    fun crash(context: String, t: Throwable) {
        Log.e(TAG, "CRASH in $context: ${t::class.java.simpleName}: ${t.message}", t)
        t.stackTrace.forEach { element ->
            Log.e(TAG, "  at ${element.className}.${element.methodName}(${element.fileName}:${element.lineNumber})")
        }
    }

    /** Log an informational lifecycle event. */
    fun info(context: String, message: String) {
        Log.i(TAG, "$context: $message")
    }

    /** Log a warning. */
    fun warn(context: String, message: String) {
        Log.w(TAG, "$context: $message")
    }

    /** Log an error that did not crash. */
    fun error(context: String, message: String) {
        Log.e(TAG, "$context: $message")
    }
}


