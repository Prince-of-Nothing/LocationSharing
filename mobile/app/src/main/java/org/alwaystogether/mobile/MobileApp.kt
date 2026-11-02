package org.alwaystogether.mobile

import android.app.Application
import org.alwaystogether.core.map.MapLibreInit
import org.alwaystogether.mobile.CrashLogger
import org.alwaystogether.mobile.CrashReport
import org.alwaystogether.mobile.di.Container

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