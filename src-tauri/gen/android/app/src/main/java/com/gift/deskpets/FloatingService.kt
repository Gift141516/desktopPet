package com.gift.deskpets

import android.app.*
import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.graphics.PixelFormat
import android.os.Build
import android.os.IBinder
import android.view.Gravity
import android.view.WindowManager
import android.webkit.*
import androidx.core.app.NotificationCompat

class FloatingService : Service() {
    private lateinit var windowManager: WindowManager
    private lateinit var webView: WebView

    override fun onBind(intent: Intent?): IBinder? = null

    // ✨ 定义一个内部类，让 JS 可以调用原生命令退出
    inner class WebAppInterface(private val context: Context) {
        @JavascriptInterface
        fun closeApp() {
            stopSelf() // 关闭服务，从而移除悬浮窗
        }
    }

    override fun onCreate() {
        super.onCreate()
        startForegroundNotification()

        windowManager = getSystemService(WINDOW_SERVICE) as WindowManager

      webView = WebView(this).apply {
          setBackgroundColor(0)
          
          // ✨ 1. 物理禁用滚动条
          isVerticalScrollBarEnabled = false
          isHorizontalScrollBarEnabled = false
          overScrollMode = WebView.OVER_SCROLL_NEVER
      
          settings.apply {
              javaScriptEnabled = true
              domStorageEnabled = true
              allowFileAccess = true
              allowFileAccessFromFileURLs = true 
              allowUniversalAccessFromFileURLs = true
              
              // ✨ 2. 核心：禁止用户缩放，并强制 1:1 比例
              setSupportZoom(false)
              builtInZoomControls = false
              displayZoomControls = false
              useWideViewPort = true
              loadWithOverviewMode = true
              // 强制不使用缓存（防止修改没生效）
              cacheMode = WebSettings.LOAD_NO_CACHE 
          }
          
          addJavascriptInterface(WebAppInterface(this@FloatingService), "Android")
          webViewClient = WebViewClient()
          loadUrl("file:///android_asset/index.html")
      }
      // 找到 layoutParams 修改如下：
      val layoutParams = WindowManager.LayoutParams().apply {
          // 稍微调大一点，确保能装下按钮和人物
          width = 400
          height = 400 
          
          type = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) 
              WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY 
          else 
              WindowManager.LayoutParams.TYPE_PHONE
          
          flags = WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or
                  WindowManager.LayoutParams.FLAG_NOT_TOUCH_MODAL or
                  WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN
          
          format = PixelFormat.TRANSLUCENT
          // 居中靠底显示
          gravity = Gravity.BOTTOM or Gravity.CENTER_HORIZONTAL
      }

        // ✨ 触摸补丁：如果点在网页透明处，允许穿透
        webView.setOnTouchListener { _, _ -> false } 

        windowManager.addView(webView, layoutParams)
    }

    private fun startForegroundNotification() {
        val channelId = "deskpets_service"
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(channelId, "桌宠服务", NotificationManager.IMPORTANCE_LOW)
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }

        val notification = NotificationCompat.Builder(this, channelId)
            .setContentTitle("桌宠运行中")
            .setContentText("点击此处管理")
            .setSmallIcon(android.R.drawable.ic_menu_info_details)
            .setPriority(NotificationCompat.PRIORITY_MIN)
            .build()
        startForeground(1, notification)
    }

    override fun onDestroy() {
        super.onDestroy()
        if (::webView.isInitialized) windowManager.removeView(webView)
    }
}