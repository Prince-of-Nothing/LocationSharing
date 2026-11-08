package org.pinpoint.mobile

import android.app.Application
import org.pinpoint.core.map.MapLibreInit
import org.pinpoint.mobile.CrashLogger
import org.pinpoint.mobile.CrashReport
import org.pinpoint.mobile.di.Container

class MobileApp : Application() {

    var container: Container? = null
        private set

    override fun onCreate() {
        super.onCreate()
        try {
            CrashLogger.info("MobileApp", "init MapLibre")
            MapLibreInit.initialize(applicationContext)
            CrashLogger.info("MobileApp", "creating Container")
            container = Container(this)
            CrashLogger.info("MobileApp", "Container ready")
        } catch (t: Throwable) {
            CrashLogger.crash("MobileApp.onCreate", t)
            CrashReport.write(this, "MobileApp.onCreate", t)
        }
    }
}