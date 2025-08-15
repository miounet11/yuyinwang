import { create } from 'zustand';

export interface TranscriptionEntry {
  id: string;
  text: string;
  timestamp: number;
  duration: number;
  model: string;
  confidence: number;
  audio_file_path?: string;
}

interface TranscriptionState {
  transcriptionText: string;
  transcriptionHistory: TranscriptionEntry[];
  selectedModel: string;
  
  setTranscription: (text: string) => void;
  setTranscriptionHistory: (history: TranscriptionEntry[]) => void;
  addTranscriptionEntry: (entry: TranscriptionEntry) => void;
  setSelectedModel: (model: string) => void;
  clearTranscription: () => void;
}

export const useTranscriptionStore = create<TranscriptionState>((set) => ({
  transcriptionText: '',
  transcriptionHistory: [],
  selectedModel: 'luyingwang-online',
  
  setTranscription: (text) => set({ transcriptionText: text }),
  setTranscriptionHistory: (history) => set({ transcriptionHistory: history }),
  addTranscriptionEntry: (entry) => set((state) => ({
    transcriptionHistory: [entry, ...state.transcriptionHistory]
  })),
  setSelectedModel: (model) => set({ selectedModel: model }),
  clearTranscription: () => set({ transcriptionText: '' })
}));