import { useState, useCallback, useEffect, useRef } from 'react';

interface MicrophoneActivityTestProps {
  deviceName: string;
}

export function MicrophoneActivityTest({ deviceName }: MicrophoneActivityTestProps) {
  const [testing, setTesting] = useState(false);
  const [volume, setVolume] = useState(0);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  
  const streamRef = useRef<MediaStream | null>(null);
  const audioCtxRef = useRef<AudioContext | null>(null);
  const animationFrameRef = useRef<number | null>(null);

  const stopTest = useCallback(() => {
    setTesting(false);
    setVolume(0);
    
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }
    if (streamRef.current) {
      streamRef.current.getTracks().forEach((track) => track.stop());
      streamRef.current = null;
    }
    if (audioCtxRef.current) {
      audioCtxRef.current.close().catch(() => {});
      audioCtxRef.current = null;
    }
  }, []);

  const startTest = useCallback(async () => {
    setErrorMsg(null);
    setTesting(true);
    setVolume(0);

    try {
      const devices = await navigator.mediaDevices.enumerateDevices();
      const audioInputs = devices.filter((d) => d.kind === 'audioinput');
      
      let targetDeviceId: string | undefined;
      if (deviceName && deviceName !== 'default') {
        const lowerName = deviceName.toLowerCase();
        const match = audioInputs.find((d) => 
          d.label.toLowerCase().includes(lowerName) || 
          lowerName.includes(d.label.toLowerCase())
        );
        if (match) {
          targetDeviceId = match.deviceId;
        }
      }

      const constraints: MediaStreamConstraints = {
        audio: targetDeviceId ? { deviceId: { exact: targetDeviceId } } : true,
      };

      const stream = await navigator.mediaDevices.getUserMedia(constraints);
      streamRef.current = stream;

      const AudioContextClass = window.AudioContext || (window as any).webkitAudioContext;
      const audioCtx = new AudioContextClass();
      audioCtxRef.current = audioCtx;

      const source = audioCtx.createMediaStreamSource(stream);
      const analyser = audioCtx.createAnalyser();
      analyser.fftSize = 256;
      source.connect(analyser);

      const bufferLength = analyser.frequencyBinCount;
      const dataArray = new Uint8Array(bufferLength);

      const updateMeter = () => {
        if (!analyser || audioCtx.state === 'closed') return;
        
        analyser.getByteFrequencyData(dataArray);
        
        let sum = 0;
        for (let i = 0; i < bufferLength; i++) {
          sum += dataArray[i];
        }
        
        const average = sum / bufferLength;
        const normalized = Math.min(100, Math.round((average / 110) * 100));
        setVolume(normalized);
        
        animationFrameRef.current = requestAnimationFrame(updateMeter);
      };

      updateMeter();
    } catch (err) {
      console.warn('Failed to start mic test:', err);
      setErrorMsg('Microphone access denied or device unavailable.');
      stopTest();
    }
  }, [deviceName, stopTest]);

  useEffect(() => {
    return () => {
      if (animationFrameRef.current) cancelAnimationFrame(animationFrameRef.current);
      if (streamRef.current) streamRef.current.getTracks().forEach((track) => track.stop());
      if (audioCtxRef.current) audioCtxRef.current.close().catch(() => {});
    };
  }, []);

  return (
    <div className="mt-5 rounded-xl bg-surface-alt border border-border p-4 animate-fade-in">
      <div className="flex items-center justify-between mb-3">
        <div>
          <h4 className="text-xs font-semibold text-text-primary uppercase tracking-wider">
            Microphone Volume Test
          </h4>
          <p className="text-[11px] text-text-muted mt-0.5">
            Test your input hardware level in real-time
          </p>
        </div>
        <button
          type="button"
          onClick={testing ? stopTest : startTest}
          className={`
            px-3 py-1.5 rounded-lg text-xs font-medium cursor-pointer transition-all duration-150
            ${testing 
              ? 'bg-danger/10 text-danger border border-danger/30 hover:bg-danger/20' 
              : 'bg-primary/10 text-primary border border-primary/20 hover:bg-primary/20'
            }
          `}
        >
          {testing ? 'Stop Test' : 'Test Microphone'}
        </button>
      </div>

      {errorMsg && (
        <p className="text-xs text-danger mb-2 font-medium">{errorMsg}</p>
      )}

      {testing && (
        <div className="space-y-2 animate-fade-in">
          <div className="h-2 w-full rounded-full bg-surface overflow-hidden relative border border-border/50">
            <div 
              className="h-full bg-gradient-to-r from-primary to-secondary transition-all duration-75 ease-out rounded-full"
              style={{ width: `${volume}%` }}
            />
          </div>
          <div className="flex justify-between text-[10px] text-text-muted font-mono">
            <span>Silent</span>
            <span className={volume > 70 ? 'text-warning font-bold' : ''}>Active</span>
          </div>
        </div>
      )}
    </div>
  );
}
