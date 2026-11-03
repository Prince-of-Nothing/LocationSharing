package org.pinpoint.mobile

import android.content.Context
import android.os.Environment
import android.util.Log
import java.io.File
import java.io.PrintWriter
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/**
 * Writes crash reports to a shareable text file on disk.
 *
 * Each crash creates a timestamped file in the app's private files directory
 * (`/data/data/org.pinpoint.mobile/files/crash_reports/`). The file
 * can be read, copied, or shared via Android's file picker / intent.
 */
object CrashReport {

    private const val TAG = "Pinpoint"
    private const val REPORT_DIR = "crash_reports"
    private const val FILENAME_FORMAT = "yyyy-MM-dd_HH-mm-ss"

    /** Append a crash entry to a new timestamped file and return the file path. */
    fun write(context: Context, contextStr: String, t: Throwable): File {
        val dir = File(Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS), REPORT_DIR)
        dir.mkdirs()
        val timestamp = SimpleDateFormat(FILENAME_FORMAT, Locale.US).format(Date())
        val file = File(dir, "crash_$timestamp.txt")
        PrintWriter(file.writer()).use { w ->
            w.println("=== Pinpoint Crash Report ===")
            w.println("Timestamp: ${Date()}")
            w.println("Context: $contextStr")
            w.println()
            w.println("Exception: ${t::class.java.name}")
            w.println("Message: ${t.message}")
            w.println()
            w.println("Stack trace:")
            t.stackTrace.forEach { w.println("  at ${it}") }
            w.println()
            w.println("Cause:")
            var cause = t.cause
            while (cause != null) {
                w.println("  - ${cause::class.java.name}: ${cause.message}")
                cause = cause.cause
            }
            w.println()
            w.println("=== End Report ===")
        }
        Log.i(TAG, "Crash report saved: ${file.absolutePath}")
        return file
    }

    /** List all saved crash reports (most recent first). */
    fun list(context: Context): List<File> {
        val dir = File(Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS), REPORT_DIR)
        return dir.listFiles()?.sortedByDescending { it.lastModified() } ?: emptyList()
    }

    /** Read a specific report file contents. */
    fun read(context: Context, file: File): String = file.readText()
}

