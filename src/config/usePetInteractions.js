import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
// 保持在外部作用域，确保全局唯一
let audioContext = null;
export function usePetInteractions() {
    const appWindow = getCurrentWindow();
    const savedAudioUrl = ref(null);

    let mediaRecorder = null;
    let audioChunks = [];
    let lastTap = 0;
    // 🌟 新增：专门用于提前唤醒音频分析器的函数
    const unlockAudio = () => {
        if (!audioContext) {
            audioContext = new (window.AudioContext || window.webkitAudioContext)();
        }
        if (audioContext.state === 'suspended') {
            audioContext.resume();
            console.log("🔓 AudioContext 已提前唤醒！");
        }
    };

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
    // 增加一个参数 customUrl
    // usePetInteractions.js 核心修复部分
    const handleAction = async (onLipSync, customUrl = null) => {
        // let audioSrc = customUrl;

        // if (!audioSrc) {
        //     const now = Date.now();
        //     if (now - lastTap < 300) {
        //         audioSrc = savedAudioUrl.value;
        //     }
        //     lastTap = now;
        // }

        // if (!audioSrc) return;
        let audioSrc = customUrl || savedAudioUrl.value;
        if (!audioSrc) return;
        try {
            const audio = new Audio();
            // audio.crossOrigin = "anonymous";
            audio.src = audioSrc;

            // 初始化 AudioContext
            if (!audioContext) {
                audioContext = new (window.AudioContext || window.webkitAudioContext)();
            }

            // 核心：在音频可以播放时再建立连接
            // audio.oncanplaythrough = async () => {
            //     try {
            //         if (audioContext.state === 'suspended') {
            //             await audioContext.resume();
            //         }

            //         // 建立分析器连接
            //         const source = audioContext.createMediaElementSource(audio);
            //         const analyser = audioContext.createAnalyser();
            //         analyser.fftSize = 128;
            //         source.connect(analyser);
            //         analyser.connect(audioContext.destination);

            //         const dataArray = new Uint8Array(analyser.frequencyBinCount);

            //         // const animate = () => {
            //         //     if (audio.paused || audio.ended) {
            //         //         onLipSync(0);
            //         //         return;
            //         //     }
            //         //     analyser.getByteFrequencyData(dataArray);
            //         //     const average = dataArray.reduce((a, b) => a + b) / dataArray.length;
            //         //     // const mouthValue = Math.min(Math.max(0, average - 30) / 30, 1.0);
            //         //     const mouthValue = Math.min(maxVolume / 80, 1.0);
            //         //     onLipSync(mouthValue);
            //         //     requestAnimationFrame(animate);
            //         // };
            //         const animate = () => {
            //             if (audio.paused || audio.ended) {
            //                 onLipSync(0);
            //                 return;
            //             }

            //             analyser.getByteFrequencyData(dataArray);

            //             // 1. 必须在这里声明 maxVolume
            //             let maxVolume = 0;

            //             // 2. 遍历找最大值
            //             for (let i = 0; i < dataArray.length; i++) {
            //                 if (dataArray[i] > maxVolume) {
            //                     maxVolume = dataArray[i];
            //                 }
            //             }

            //             // 3. 计算嘴型张合度 (如果觉得嘴巴张太大/太小，微调这个 80)
            //             const mouthValue = Math.min(maxVolume / 80, 1.0);

            //             onLipSync(mouthValue);
            //             requestAnimationFrame(animate);
            //         };

            //         await audio.play();
            //         animate();
            //         console.log("✅ 播放启动成功");
            //     } catch (err) {
            //         console.error("连接分析器或播放失败:", err);
            //     }
            // };
            // 核心：在音频可以播放时再建立连接
            audio.oncanplaythrough = async () => {
                try {
                    if (audioContext.state === 'suspended') {
                        await audioContext.resume();
                    }

                    const source = audioContext.createMediaElementSource(audio);
                    const analyser = audioContext.createAnalyser();
                    analyser.fftSize = 128;
                    source.connect(analyser);
                    analyser.connect(audioContext.destination);

                    const dataArray = new Uint8Array(analyser.frequencyBinCount);

                    // 🌟 新增：记录上一帧的嘴型值，用于平滑过渡
                    let lastMouthValue = 0;
                    let frameCount = 0; // 帧计数器，用来控制打印频率
                    // 现在判断当前数值为155,解决了嘴巴一直张开的原因
                    const volumeThreshold = 155;

                    const animate = () => {
                        if (audio.paused || audio.ended) {
                            onLipSync(0);
                            return;
                        }

                        analyser.getByteFrequencyData(dataArray);

                        let maxVolume = 0;
                        for (let i = 0; i < dataArray.length; i++) {
                            if (dataArray[i] > maxVolume) {
                                maxVolume = dataArray[i];
                            }
                        }
                        frameCount++;
                        if (frameCount % 10 === 0) {
                            console.log(`🔊 实时原始音量 (0-255): ${maxVolume}`);
                        }
                        // 🌟 1. 动态底噪消除：把你那高达 150 的背景数值直接砍掉！
                        // 只要声音小于 155，统统视为静音 (等于 0)
                        let adjustedVolume = Math.max(0, maxVolume - volumeThreshold);

                        // 🌟 2. 精准张合度：砍掉底噪后，剩下的有效音量最高大概在 100 左右
                        // 所以我们除以 80，让嘴巴能随着声音大小自然张合
                        let targetMouthValue = Math.min(adjustedVolume / 80, 1.0);

                        // 3. 平滑过渡，像真人一样连贯
                        lastMouthValue = lastMouthValue + (targetMouthValue - lastMouthValue) * 0.4;

                        // 4. 彻底归零保险
                        if (lastMouthValue < 0.05) {
                            lastMouthValue = 0;
                        }

                        onLipSync(lastMouthValue);
                        requestAnimationFrame(animate);
                    };

                    // 🌟 终极保险：监听音频原生的结束事件，强制闭嘴
                    audio.onended = () => {
                        onLipSync(0);
                    };

                    await audio.play();
                    animate();
                    console.log("✅ 播放启动成功，平滑口型已开启");
                } catch (err) {
                    console.error("连接分析器或播放失败:", err);
                }
            };

            audio.onerror = () => {
                console.error("音频加载失败，错误码:", audio.error.code);
            };

        } catch (err) {
            console.error("handleAction 外部捕获失败:", err);
        }
    };
    return {
        setIgnoreMouse,
        handleDrag,
        startRecording,
        stopRecording,
        handleAction,
        unlockAudio,
        savedAudioUrl,
    };
}