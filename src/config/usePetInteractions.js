import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

export function usePetInteractions() {
    const appWindow = getCurrentWindow();
    const savedAudioUrl = ref(null);

    let mediaRecorder = null;
    let audioChunks = [];
    let lastTap = 0;

    // --- 窗口控制 ---
    const setIgnoreMouse = async (ignore) => {
        // await invoke('set_ignore_mouse', { ignore });
    };

    const handleDrag = async (e) => {
        // if (e.button === 0) appWindow.startDragging();
        try {
            await appWindow.startDragging();
        } catch (err) {
            console.error("系统拖拽启动失败:", err);
        }
    };

    // --- 录音逻辑 ---
    const startRecording = async () => {
        try {
            const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
            mediaRecorder = new MediaRecorder(stream);
            audioChunks = [];
            mediaRecorder.ondataavailable = (e) => audioChunks.push(e.data);
            mediaRecorder.start();
            console.log("🎤 录音开始...");
        } catch (err) {
            console.error("麦克风权限开启失败", err);
        }
    };

    const stopRecording = () => {
        return new Promise((resolve) => {
            if (!mediaRecorder) return resolve(null);
            mediaRecorder.onstop = () => {
                const audioBlob = new Blob(audioChunks, { type: 'audio/wav' });
                const url = URL.createObjectURL(audioBlob);
                savedAudioUrl.value = url;
                resolve(url);
            };
            mediaRecorder.stop();
            console.log("✅ 录音已保存");
        });
    };

    // --- 播放逻辑 ---
    // const handleAction = () => {
    //     const now = Date.now();
    //     if (now - lastTap < 300) { // 双击触发播放
    //         if (savedAudioUrl.value) {
    //             new Audio(savedAudioUrl.value).play();
    //         } else {
    //             console.log("你好呀！");
    //         }
    //     }
    //     lastTap = now;
    // };
    let audioContext = null;
    let analyser = null;
    let dataArray = null;
    let animationId = null;
    const handleAction = async(onLipSync) => {
        const now = Date.now();
        if (now - lastTap < 300) {
            if (savedAudioUrl.value) {
                const audio = new Audio(savedAudioUrl.value);

                // 初始化 AudioContext 用于频谱分析
                if (!audioContext) {
                    audioContext = new (window.AudioContext || window.webkitAudioContext)();
                }
                if (audioContext.state === 'suspended') {
                    await audioContext.resume();
                }
                const source = audioContext.createMediaElementSource(audio);
                analyser = audioContext.createAnalyser();
                analyser.fftSize = 128; // 较小的采样率足够口型使用
                source.connect(analyser);
                analyser.connect(audioContext.destination);

                dataArray = new Uint8Array(analyser.frequencyBinCount);

                // usePetInteractions.js 里的 animate 函数片段
                // usePetInteractions.js 里的 handleAction 内部
                const animate = () => {
                    // 检查音频是否结束
                    if (audio.paused || audio.ended) {
                        cancelAnimationFrame(animationId);
                        animationId = null;
                        // 核心：强制发送 0 信号
                        onLipSync(0);
                        console.log("音频播放结束，强制闭嘴");
                        return;
                    }

                    analyser.getByteFrequencyData(dataArray);
                    const voiceData = dataArray.slice(0, dataArray.length / 2);
                    let average = voiceData.reduce((a, b) => a + b) / voiceData.length;

                    // 提高过滤门槛，防止最后的电流声让嘴巴停在微张状态
                    const noiseFloor = 40;
                    const sensitiveVolume = Math.max(0, average - noiseFloor);
                    const mouthValue = Math.min(sensitiveVolume / 30, 1.0);

                    onLipSync(mouthValue);
                    animationId = requestAnimationFrame(animate);
                };

                await audio.play();
                animate();
            }
        }
        lastTap = now;
    };

    return {
        setIgnoreMouse,
        handleDrag,
        startRecording,
        stopRecording,
        handleAction,
        savedAudioUrl
    };
}