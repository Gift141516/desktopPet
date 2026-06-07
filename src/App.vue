<script setup>
import { usePetInteractions } from './config/usePetInteractions';
import Live2dViewer from "./views/Live2dViewer.vue";
import ContextMenu from "./components/ContextMenu.vue";
import { onMounted, ref, onUnmounted } from "vue";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
// 侧边栏状态管理
const isSideMode = ref(false);  // 是否启用侧边栏模式
const isExpanded = ref(false);   // 在侧边栏模式下，当前是否展开
const {
  handleDrag, startRecording,
  stopRecording, handleAction,
  unlockAudio
} = usePetInteractions();
const viewerRef = ref(null);
let pressTimer = null;
let isRecording = false;
let startTimestamp = 0;
let startPos = { x: 0, y: 0 };
let isDraggingTriggered = ref(false); // 标记是否已经开启了系统拖拽
const isRecordingStatus = ref(false); // 新增：用于界面显示
let unlistenMenu = null; // 存起来
const contextMenuRef = ref(null); // 右键菜单组件引用

// 右键事件
const onRightClick = async (e) => {
  // 1. 阻止浏览器默认的右键菜单
  if (e.preventDefault) e.preventDefault();

  // 🌟 核心破解：在弹出菜单之前，利用这次宝贵的网页点击，直接唤醒分析器！
  if (typeof unlockAudio === 'function') {
    unlockAudio();
  }

  // 2. 显示自定义右键菜单
  if (contextMenuRef.value) {
    // 使用原生事件的坐标，如果是 PIXI 事件则使用 clientX/Y
    const x = e.clientX || e.data?.global?.x || 0;
    const y = e.clientY || e.data?.global?.y || 0;
    contextMenuRef.value.show(x, y);
  }
};
const onDown = (e) => {
  if (e.button !== 0) return;

  startTimestamp = Date.now();
  startPos = { x: e.screenX, y: e.screenY };
  isRecording = false;
  isDraggingTriggered.value = false;

  // 1. 延迟判定录音：只有在没发生拖拽的情况下才开启录音
  pressTimer = setTimeout(() => {
    if (!isDraggingTriggered.value) {
      isRecording = true;
      startRecording();
    }
  }, 500);
};

// 监听移动：如果移动距离超过阈值，则视为拖拽
const onMove = (e) => {
  // if (isRecording || isDraggingTriggered.value || startPos.x === 0) return;
  //
  // const dx = e.screenX - startPos.x;
  // const dy = e.screenY - startPos.y;
  //
  // // 移动超过 5 像素则触发系统拖拽
  // if (Math.abs(dx) > 5 || Math.abs(dy) > 5) {
  //   isDraggingTriggered.value = true;
  //   clearTimeout(pressTimer); // 移动了就不录音了
  //   handleDrag(e);
  // }
  // 确保能获取到坐标，PIXI 事件通常在 e.data.global 或 nativeEvent 中
  const screenX = e.screenX || e.data?.global?.x;
  const screenY = e.screenY || e.data?.global?.y;

  if (isRecording || isDraggingTriggered.value || !startPos.x || !screenX) return;

  const dx = screenX - startPos.x;
  const dy = screenY - startPos.y;

  if (Math.abs(dx) > 5 || Math.abs(dy) > 5) {
    console.log("检测到移动，触发拖拽", dx, dy);
    isDraggingTriggered.value = true;
    clearTimeout(pressTimer);

    // 🔥 关键修复：拖动时自动退出侧边栏模式
    if (isSideMode.value) {
      // 只更新状态，不重复调用后端（避免与展开动画冲突）
      isSideMode.value = false;
      isExpanded.value = false;
    }

    handleDrag(e);
  }
};

const onUp = async () => {
  clearTimeout(pressTimer);
  const duration = Date.now() - startTimestamp;
  const finalStartTimestamp = startTimestamp;
  startTimestamp = 0; // 重置

  if (isDraggingTriggered.value) {
    // 如果已经触发了拖拽，系统会接管 mouseup，这里的逻辑可能不会执行
    return;
  }

  if (isRecording) {
    // 情况 A: 结束录音
    await stopRecording();
    isRecording = false;
  } else if (duration < 300 && duration > 0) {
    // 情况 B: 判定为快速点击
    // handleAction();
    await handleAction((val) => {
      viewerRef.value?.lipSync(val);
    });
  }
};
const onPointerOver = () => {
  console.log("响应：关闭穿透");
  // setIgnoreMouse(false);
};

const onPointerOut = () => {
  // 延迟一小会儿再穿透，防止微小抖动导致的失焦
  setTimeout(() => {
    if (!isRecording && !isDraggingTriggered.value) {
      console.log("响应：开启穿透");
      // setIgnoreMouse(true);
    }
  }, 100);
};

const onMenuWeather = async () => {
  try {
    const text = "今天天气不错，要不要出去走走？";

    // 1. 这里的变量名建议直接叫 audioBytes，因为它现在是字节流而不是路径了
    const audioBytes = await invoke('generate_tts', { text });
    console.log("成功拿到语音字节流，长度:", audioBytes.length);

    // 2. 使用正确的变量名 audioBytes 来创建 Blob
    const audioBlob = new Blob([new Uint8Array(audioBytes)], { type: 'audio/wav' });

    // 3. 生成浏览器安全信任的 URL
    const audioUrl = URL.createObjectURL(audioBlob);

    // 4. 播放并触发同步
    await handleAction((val) => {
      viewerRef.value?.lipSync(val);
    }, audioUrl);

    // 延时释放内存
    setTimeout(() => URL.revokeObjectURL(audioUrl), 10000);
  } catch (err) {
    console.error("语音生成失败:", err);
  }
};
// 核心逻辑：切换窗口侧边状态
const toggleSide = async (enable) => {
  isSideMode.value = enable;
  if (enable) {
    // 启用侧边栏模式：收缩到边缘
    isExpanded.value = false;
    await invoke("toggle_side_status", { isHide: true });
  } else {
    // 禁用侧边栏模式：完全展开
    isExpanded.value = false;
    await invoke("toggle_side_status", { isHide: false });
  }
};

// 鼠标划入标签：展开窗口
const onTabEnter = async () => {
  console.log("鼠标进入标签，展开窗口");
  isExpanded.value = true;
  await invoke("toggle_side_status", { isHide: false });
};

// 鼠标离开内容区：收缩窗口
const onContentLeave = async () => {
  if (isSideMode.value && isExpanded.value) {
    console.log("鼠标离开内容，收缩窗口");
    isExpanded.value = false;
    await invoke("toggle_side_status", { isHide: true });
  }
};


onMounted(async () => {
  // 不再需要监听 Rust 菜单事件，因为现在使用自定义菜单
  // unlistenMenu = await listen('menu-action', async (event) => {
  //   const action = event.payload;
  //   if (action === 'weather') {
  //     console.log("用户点击了：今天天气怎么样");
  //     onMenuWeather();
  //   } else if (action === 'chat') {
  //     console.log("用户点击了：陪我聊聊天");
  //   } else if (action === 'hide') {
  //     console.log("执行隐藏到侧边");
  //     await toggleSide(true);
  //   }
  // });
});

// 处理自定义菜单的点击事件
const handleMenuAction = async (action) => {
  if (action === 'weather') {
    console.log("用户点击了：今天天气怎么样");
    await onMenuWeather();
  } else if (action === 'chat') {
    console.log("用户点击了：陪我聊聊天");
  } else if (action === 'hide') {
    console.log("执行隐藏到侧边");
    await toggleSide(true);
  } else if (action === 'quit') {
    console.log("退出程序");
    await invoke('exit_app');
  }
};

onUnmounted(() => {
  // 组件销毁时，注销监听，防止重复触发
  if (unlistenMenu) unlistenMenu();
});
</script>

<template>
  <div class="main-app">
    <!-- 吸附标签：只在侧边栏模式且未展开时显示 -->
    <div
      v-if="isSideMode && !isExpanded"
      class="edge-tab"
      @mouseenter="onTabEnter">
      <div class="tab-icon">👻</div>
    </div>

    <!-- 主内容区 -->
    <div
      class="content-area"
      :class="{ 'is-hidden': isSideMode && !isExpanded }"
      @mouseleave="onContentLeave">
      <Live2dViewer ref="viewerRef" modelPath="model/runtime/kei_basic_free.model3.json" @pointerover="onPointerOver"
        @pointerout="onPointerOut" @pointerdown="onDown" @pointermove="onMove" @pointerup="onUp"
        @contextmenu="onRightClick" />

      <!-- 自定义右键菜单 -->
      <ContextMenu ref="contextMenuRef" @menu-action="handleMenuAction" />
    </div>
  </div>
</template>

<style>
/* 彻底禁止点击时的所有高亮和轮廓 */
* {
  -webkit-tap-highlight-color: transparent;
  outline: none !important;
  user-select: none;
  /* 防止长按选中文本蓝色高亮 */
  margin: 0;
  padding: 0;
}

canvas {
  outline: none !important;
  border: none !important;
  box-shadow: none !important;
  /* 彻底移除这个该死的模糊阴影 */
  filter: none !important;
  /* 防止使用了 drop-shadow 滤镜 */
  border-radius: 0 !important;
  /* 清除可能存在的圆角 */
  background: transparent !important;
  /* 确保背景彻底透明 */
}

html,
body,
#app {
  margin: 0;
  padding: 0;
  width: 450px;
  height: 600px;
  background: transparent !important;
  overflow: hidden;
  /* 关键：PC端这里必须为 auto，否则JS永远收不到 hover 事件 */
  pointer-events: none;
  /* 全局穿透 */
}

.main-app {
  width: 100%;
  height: 100%;
  position: relative;
}

/* 吸附标签样式 */
.edge-tab {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 30px;
  height: 80px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 0 8px 8px 0;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  pointer-events: auto;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.2);
  transition: all 0.3s ease;
  z-index: 9999;
}

.edge-tab:hover {
  width: 35px;
  box-shadow: 4px 0 12px rgba(0, 0, 0, 0.3);
}

.tab-icon {
  font-size: 24px;
  animation: float 2s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-5px); }
}

/* 内容区域 */
.content-area {
  width: 100%;
  height: 100%;
  transition: opacity 0.2s ease;
}

.content-area.is-hidden {
  opacity: 0;
  pointer-events: none;
}
</style>