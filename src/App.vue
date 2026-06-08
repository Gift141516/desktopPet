<script setup>
import { usePetInteractions } from './config/usePetInteractions';
import Live2dViewer from "./views/Live2dViewer.vue";
import ContextMenu from "./components/ContextMenu.vue";
import { onMounted, ref, onUnmounted, onBeforeUnmount } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { currentMonitor } from "@tauri-apps/api/window";
// 侧边栏状态管理
const isSideMode = ref(false);  // 是否启用侧边栏模式
const isExpanded = ref(false);   // 在侧边栏模式下，当前是否展开
const currentEdge = ref("right"); // 当前吸附的边缘：left/right/top/bottom
const ENABLE_AUTO_EDGE_DOCK = false;
let mouseInContent = false; // 🔥 鼠标是否真正进入了内容区
let pointerInContent = false;
let isContextMenuOpen = false;
let isPointerInContextMenu = false;
let isWindowMoving = false; // 🔥 窗口是否正在移动中
let pendingCollapse = false; // 🔥 窗口移动期间是否有待处理的收缩请求
let isProgrammaticOperation = false; // 🔥 标记程序触发的窗口操作（vs 用户拖拽）
let collapseTimer = null; // 🔥 收缩延迟定时器
let menuCollapseTimer = null;
let isExpanding = false; // 🔥 标记正在展开中（展开保护期）

const clearCollapseTimer = () => {
  if (collapseTimer) {
    clearTimeout(collapseTimer);
    collapseTimer = null;
  }
};

const clearMenuCollapseTimer = () => {
  if (menuCollapseTimer) {
    clearTimeout(menuCollapseTimer);
    menuCollapseTimer = null;
  }
};

const collapseToSide = async () => {
  if (!isSideMode.value || !isExpanded.value) return;

  console.log("✅ 执行收缩");
  clearMenuCollapseTimer();
  contextMenuRef.value?.closeMenu();
  isContextMenuOpen = false;
  isPointerInContextMenu = false;
  isExpanded.value = false;
  mouseInContent = false;
  pointerInContent = false;
  pendingCollapse = false;

  isProgrammaticOperation = true;
  try {
    await invoke("toggle_side_status", { isHide: true, edge: currentEdge.value });
  } finally {
    setTimeout(() => {
      isProgrammaticOperation = false;
      console.log("✅ 收缩完成，解除程序操作标记");
    }, 500);
  }
};

const scheduleCollapse = (delay = 200, requireMouseInContent = true) => {
  clearCollapseTimer();
  collapseTimer = setTimeout(async () => {
    collapseTimer = null;
    if (isContextMenuOpen) return;
    if (requireMouseInContent && !mouseInContent) return;
    await collapseToSide();
  }, delay);
};
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

const handleContextMenuOpen = () => {
  isContextMenuOpen = true;
  isPointerInContextMenu = false;
  mouseInContent = true;
  pointerInContent = true;
  clearCollapseTimer();
  clearMenuCollapseTimer();
};

const handleContextMenuClose = () => {
  isContextMenuOpen = false;
  isPointerInContextMenu = false;
  clearMenuCollapseTimer();
  if (isSideMode.value && isExpanded.value && !pointerInContent) {
    scheduleCollapse(200, false);
  }
};

const scheduleMenuCloseAndCollapse = (delay = 250) => {
  clearMenuCollapseTimer();
  menuCollapseTimer = setTimeout(async () => {
    menuCollapseTimer = null;
    if (!isContextMenuOpen || isPointerInContextMenu || pointerInContent) return;

    contextMenuRef.value?.closeMenu();
    isContextMenuOpen = false;
    isPointerInContextMenu = false;
    await collapseToSide();
  }, delay);
};

const handleContextMenuEnter = () => {
  isPointerInContextMenu = true;
  clearMenuCollapseTimer();
  clearCollapseTimer();
};

const handleContextMenuLeave = () => {
  isPointerInContextMenu = false;
  if (isSideMode.value && isExpanded.value && !pointerInContent) {
    scheduleMenuCloseAndCollapse(250);
  }
};

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
    handleContextMenuOpen();
    // 使用原生事件的坐标，如果是 PIXI 事件则使用 clientX/Y
    const x = e.clientX ?? e.data?.global?.x ?? 0;
    const y = e.clientY ?? e.data?.global?.y ?? 0;
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
  const screenX = e.screenX ?? e.data?.global?.x;
  const screenY = e.screenY ?? e.data?.global?.y;

  if (isRecording || isDraggingTriggered.value || startTimestamp === 0 || screenX == null || screenY == null) return;

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
    if (ENABLE_AUTO_EDGE_DOCK) {
      // 拖拽结束，检查是否需要自动吸附到边缘
      console.log("🔥 拖拽结束，开始检查自动吸附...");
      await checkAutoEdgeDock();
    }
    isDraggingTriggered.value = false;
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

// 🔥 智能边缘吸附检测
const checkAutoEdgeDock = async () => {
  if (!ENABLE_AUTO_EDGE_DOCK) return;

  try {
    const currentWindow = getCurrentWindow();

    // 获取窗口位置和屏幕尺寸
    const position = await currentWindow.outerPosition();
    const size = await currentWindow.outerSize();
    const monitor = await currentMonitor();

    console.log("📍 当前窗口位置:", position);
    console.log("📐 窗口尺寸:", size);
    console.log("🖥️ 屏幕尺寸:", monitor?.size);

    if (!monitor) {
      console.warn("⚠️ 无法获取显示器信息");
      return;
    }

    const screenWidth = monitor.size.width;
    const screenHeight = monitor.size.height;
    const halfWidth = size.width * 0.5;
    const halfHeight = size.height * 0.5;

    console.log(`🎯 检测阈值: halfWidth=${halfWidth}, halfHeight=${halfHeight}`);

    // 判断靠近哪个边缘（一半在屏幕外就吸附）
    let edge = null;

    if (position.x < -halfWidth) {
      // 左边缘：窗口左边界移出屏幕超过一半宽度
      edge = "left";
      console.log(`✅ 触发左边缘: position.x(${position.x}) < -halfWidth(${-halfWidth})`);
    } else if (position.x > screenWidth - halfWidth) {
      // 右边缘：窗口左边界超过屏幕右边界 - 一半宽度
      edge = "right";
      console.log(`✅ 触发右边缘: position.x(${position.x}) > screenWidth - halfWidth(${screenWidth - halfWidth})`);
    } else if (position.y < -halfHeight) {
      // 上边缘：窗口上边界移出屏幕超过一半高度
      edge = "top";
      console.log(`✅ 触发上边缘: position.y(${position.y}) < -halfHeight(${-halfHeight})`);
    } else if (position.y > screenHeight - halfHeight) {
      // 下边缘：窗口上边界超过屏幕下边界 - 一半高度
      edge = "bottom";
      console.log(`✅ 触发下边缘: position.y(${position.y}) > screenHeight - halfHeight(${screenHeight - halfHeight})`);
    } else {
      console.log("❌ 未触发任何边缘");
    }

    if (edge) {
      console.log(`🎉 检测到靠近 ${edge} 边缘，自动吸附`);
      currentEdge.value = edge;
      isSideMode.value = true;
      isExpanded.value = false;
      await invoke("toggle_side_status", { isHide: true, edge });
    }
  } catch (err) {
    console.error("边缘检测失败:", err);
  }
};

// 核心逻辑：切换窗口侧边状态
const toggleSide = async (enable) => {
  isSideMode.value = enable;
  isProgrammaticOperation = true;
  try {
    if (enable) {
      // 启用侧边栏模式：收缩到边缘
      isExpanded.value = false;
      await invoke("toggle_side_status", { isHide: true, edge: currentEdge.value });
    } else {
      // 禁用侧边栏模式：完全展开
      isExpanded.value = false;
      await invoke("toggle_side_status", { isHide: false, edge: currentEdge.value });
    }
  } finally {
    setTimeout(() => {
      isProgrammaticOperation = false;
    }, 500);
  }
};

// 鼠标划入标签：展开窗口
const onTabEnter = async () => {
  if (!isSideMode.value || isExpanded.value || isExpanding) return;

  console.log("🟢 [1] 标签 mouseenter - 开始展开");
  console.log(`   当前状态: isExpanded=${isExpanded.value}, mouseInContent=${mouseInContent}`);

  // 🔥 清除收缩定时器（鼠标回来了）
  clearCollapseTimer();

  mouseInContent = false; // 🔥 重置标记（还没进入内容区）
  isExpanded.value = true;

  // 🔥 进入展开保护期
  isExpanding = true;

  // 🔥 标记程序操作开始
  isProgrammaticOperation = true;
  await invoke("toggle_side_status", { isHide: false, edge: currentEdge.value });

  // 🔥 等待展开动画完全完成（800ms = 动画时长 + 缓冲）
  setTimeout(() => {
    isProgrammaticOperation = false;
    isExpanding = false; // 🔥 解除展开保护期
    console.log("🟢 [2] 标签展开完成，解除保护期");
    if (isSideMode.value && isExpanded.value) {
      scheduleCollapse(700, false);
    }
  }, 800);
};

// 🔥 鼠标进入内容区：允许后续收缩
const onContentEnter = () => {
  console.log("🔵 [3] 内容区 mouseenter - 允许收缩");
  console.log(`   当前状态: isExpanded=${isExpanded.value}, mouseInContent=${mouseInContent}`);

  if (isExpanding) return;

  pointerInContent = true;

  // 🔥 清除收缩定时器（鼠标回来了）
  clearCollapseTimer();

  mouseInContent = true;
};

const onContentMove = () => {
  if (!isSideMode.value || !isExpanded.value || isExpanding) return;

  pointerInContent = true;
  if (!mouseInContent) {
    mouseInContent = true;
  }
  clearCollapseTimer();
};

// 鼠标离开内容区：收缩窗口
const onContentLeave = async () => {
  console.log("🔴 [4] 内容区 mouseleave - 尝试收缩");
  console.log(`   当前状态: isSideMode=${isSideMode.value}, isExpanded=${isExpanded.value}, mouseInContent=${mouseInContent}, isWindowMoving=${isWindowMoving}, isExpanding=${isExpanding}`);

  // 🔥 展开保护期内，直接忽略所有收缩请求
  if (isExpanding) {
    console.log("🚫 正在展开中，忽略收缩");
    return;
  }

  pointerInContent = false;

  if (isContextMenuOpen) {
    console.log("🚫 右键菜单打开中，等待菜单 hover 状态");
    scheduleMenuCloseAndCollapse(250);
    return;
  }

  // 🔥 窗口移动期间记录待处理的收缩请求（只有展开状态才需要延迟收缩）
  if (isWindowMoving && isExpanded.value) {
    console.log("🚫 窗口正在移动，延迟收缩请求");
    pendingCollapse = true;
    return;
  }

  if (isSideMode.value && isExpanded.value) {
    // 🔥 只有鼠标真正进入过内容区，才允许收缩
    if (!mouseInContent) {
      console.log("🚫 鼠标从未进入内容区，忽略收缩请求");
      return;
    }

    // 🔥 延迟200ms收缩，避免鼠标快速划过时抖动
    scheduleCollapse(200, true);
  }
};


let moveEndTimer = null; // 用于检测窗口移动结束
let unlistenMove = null; // 🔥 保存 unlisten 函数，用于清理监听器

onMounted(async () => {
  // 监听窗口位置变化
  unlistenMove = await listen('tauri://move', async (event) => {
    if (!ENABLE_AUTO_EDGE_DOCK && !isSideMode.value) return;

    // 🔥 如果是程序触发的窗口操作，直接忽略（不进入防抖逻辑）
    if (isProgrammaticOperation) {
      console.log("🚫 程序操作触发的move事件，忽略");
      return;
    }

    console.log("🚚 用户拖拽窗口，进入移动检测");
    isWindowMoving = true; // 🔥 标记窗口正在移动

    // 清除之前的定时器
    if (moveEndTimer) clearTimeout(moveEndTimer);

    // 300ms 没有新的移动事件就认为移动结束
    moveEndTimer = setTimeout(async () => {
      console.log("🛑 窗口移动结束");
      isWindowMoving = false; // 🔥 移动结束
      if (ENABLE_AUTO_EDGE_DOCK) {
        await checkAutoEdgeDock();
      }

      // 🔥 如果移动期间有待处理的收缩请求，现在执行（确保窗口还是展开状态）
      if (pendingCollapse && mouseInContent && isExpanded.value) {
        console.log("⏳ 执行延迟的收缩请求");
        pendingCollapse = false;
        isExpanded.value = false;
        mouseInContent = false;
        pointerInContent = false;

        // 🔥 标记程序操作
        isProgrammaticOperation = true;
        await invoke("toggle_side_status", { isHide: true, edge: currentEdge.value });

        setTimeout(() => {
          isProgrammaticOperation = false;
          console.log("✅ 延迟收缩完成，解除程序操作标记");
        }, 500);
      }
    }, 300);
  });
});

onBeforeUnmount(() => {
  // 🔥 清理事件监听器，防止热更新时重复注册
  if (unlistenMove) {
    unlistenMove();
    unlistenMove = null;
  }
  if (moveEndTimer) {
    clearTimeout(moveEndTimer);
    moveEndTimer = null;
  }
  if (collapseTimer) {
    clearTimeout(collapseTimer);
    collapseTimer = null;
  }
  clearMenuCollapseTimer();
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
    // 🔥 修复：获取当前窗口位置，判断最近的边缘
    const currentWindow = getCurrentWindow();
    const position = await currentWindow.outerPosition();
    const size = await currentWindow.outerSize();
    const monitor = await currentMonitor();

    if (monitor) {
      const screenWidth = monitor.size.width;
      const screenHeight = monitor.size.height;

      // 判断窗口中心点靠近哪个边缘
      const centerX = position.x + size.width / 2;
      const centerY = position.y + size.height / 2;

      const distToLeft = centerX;
      const distToRight = screenWidth - centerX;
      const distToTop = centerY;
      const distToBottom = screenHeight - centerY;

      const minDist = Math.min(distToLeft, distToRight, distToTop, distToBottom);

      if (minDist === distToLeft) currentEdge.value = "left";
      else if (minDist === distToRight) currentEdge.value = "right";
      else if (minDist === distToTop) currentEdge.value = "top";
      else currentEdge.value = "bottom";

      console.log(`最近边缘：${currentEdge.value}`);
    }

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
      :class="[`edge-${currentEdge}`]"
      @mouseenter="onTabEnter">
      <div class="tab-icon">👻</div>
    </div>

    <!-- 主内容区 -->
    <div
      class="content-area"
      :class="{ 'is-hidden': isSideMode && !isExpanded }"
      @mouseenter="onContentEnter"
      @mousemove="onContentMove"
      @mouseleave="onContentLeave">
      <Live2dViewer ref="viewerRef" modelPath="model/runtime/kei_basic_free.model3.json" @pointerover="onPointerOver"
        @pointerout="onPointerOut" @pointerdown="onDown" @pointermove="onMove" @pointerup="onUp"
        @contextmenu="onRightClick" />

      <!-- 自定义右键菜单 -->
      <ContextMenu
        ref="contextMenuRef"
        @menu-action="handleMenuAction"
        @menu-open="handleContextMenuOpen"
        @menu-close="handleContextMenuClose"
        @menu-enter="handleContextMenuEnter"
        @menu-leave="handleContextMenuLeave" />
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
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  pointer-events: auto;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  transition: all 0.3s ease;
  z-index: 9999;
}

/* 左边缘：标签在窗口右侧（靠近屏幕内） */
.edge-tab.edge-left {
  right: 0;  /* 🔥 修复：改为 right，标签才能在屏幕内可见 */
  top: 50%;
  transform: translateY(-50%);
  width: 30px;
  height: 80px;
  border-radius: 0 8px 8px 0;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.2);
}

.edge-tab.edge-left:hover {
  width: 35px;
  box-shadow: 4px 0 12px rgba(0, 0, 0, 0.3);
}

/* 右边缘：标签在窗口左侧（靠近屏幕内） */
.edge-tab.edge-right {
  left: 0;  /* 🔥 修复：改为 left，标签才能在屏幕内可见 */
  top: 50%;
  transform: translateY(-50%);
  width: 30px;
  height: 80px;
  border-radius: 8px 0 0 8px;
  box-shadow: -2px 0 8px rgba(0, 0, 0, 0.2);
}

.edge-tab.edge-right:hover {
  width: 35px;
  box-shadow: -4px 0 12px rgba(0, 0, 0, 0.3);
}

/* 上边缘：标签在窗口下侧（靠近屏幕内） */
.edge-tab.edge-top {
  left: 50%;
  bottom: 0;  /* 🔥 修复：改为 bottom，标签才能在屏幕内可见 */
  transform: translateX(-50%);
  width: 80px;
  height: 30px;
  border-radius: 0 0 8px 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.edge-tab.edge-top:hover {
  height: 35px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

/* 下边缘：标签在窗口上侧（靠近屏幕内） */
.edge-tab.edge-bottom {
  left: 50%;
  top: 0;  /* 🔥 修复：改为 top，标签才能在屏幕内可见 */
  transform: translateX(-50%);
  width: 80px;
  height: 30px;
  border-radius: 8px 8px 0 0;
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.2);
}

.edge-tab.edge-bottom:hover {
  height: 35px;
  box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.3);
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
  transition: opacity 0.8s ease; /* 🔥 改为 0.8s，匹配后端 800ms 展开时长 */
  will-change: opacity; /* 🔥 GPU 加速，减少重绘开销 */
  transform: translateZ(0); /* 🔥 强制 GPU 合成 */
  pointer-events: auto; /* 🔥 确保可以接收鼠标事件（mouseleave） */
}

.content-area.is-hidden {
  opacity: 0;
  pointer-events: none;
}
</style>
