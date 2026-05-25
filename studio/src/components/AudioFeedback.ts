type AudioWindow = Window &
  typeof globalThis & {
    webkitAudioContext?: typeof AudioContext;
  };

let audioContext: AudioContext | null = null;

export function playMicroClick(enabled: boolean): void {
  if (!enabled) {
    return;
  }

  try {
    const contextConstructor = window.AudioContext ?? (window as AudioWindow).webkitAudioContext;
    if (!contextConstructor) {
      return;
    }
    audioContext ??= new contextConstructor();
    const now = audioContext.currentTime;
    const oscillator = audioContext.createOscillator();
    const gain = audioContext.createGain();
    const filter = audioContext.createBiquadFilter();

    oscillator.type = "sine";
    oscillator.frequency.setValueAtTime(1320, now);
    oscillator.frequency.exponentialRampToValueAtTime(860, now + 0.028);
    filter.type = "highpass";
    filter.frequency.setValueAtTime(520, now);
    gain.gain.setValueAtTime(0.0001, now);
    gain.gain.exponentialRampToValueAtTime(0.032, now + 0.006);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.034);

    oscillator.connect(filter);
    filter.connect(gain);
    gain.connect(audioContext.destination);
    oscillator.start(now);
    oscillator.stop(now + 0.04);
  } catch {
    // Audio is an optional tactile enhancement. Browser/Tauri audio blocks should never interrupt the UI.
  }
}
