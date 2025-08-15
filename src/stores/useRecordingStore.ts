import { create } from 'zustand';

interface RecordingState {
  isRecording: boolean;
  recordingDuration: number;
  audioLevel: number;
  audioData: Float32Array | null;
  
  setIsRecording: (recording: boolean) => void;
  setRecordingDuration: (duration: number) => void;
  setAudioLevel: (level: number) => void;
  setAudioData: (data: Float32Array | null) => void;
  resetRecording: () => void;
}

export const useRecordingStore = create<RecordingState>((set) => ({
  isRecording: false,
  recordingDuration: 0,
  audioLevel: 0,
  audioData: null,
  
  setIsRecording: (recording) => set({ isRecording: recording }),
  setRecordingDuration: (duration) => set({ recordingDuration: duration }),
  setAudioLevel: (level) => set({ audioLevel: level }),
  setAudioData: (data) => set({ audioData: data }),
  
  resetRecording: () => set({
    isRecording: false,
    recordingDuration: 0,
    audioLevel: 0,
    audioData: null
  })
}));