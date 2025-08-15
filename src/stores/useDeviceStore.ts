import { create } from 'zustand';

export interface AudioDevice {
  name: string;
  id: string;
  is_default: boolean;
  is_available: boolean;
}

interface DeviceState {
  audioDevices: AudioDevice[];
  selectedDevice: string | null;
  
  setDevices: (devices: AudioDevice[]) => void;
  setSelectedDevice: (deviceId: string | null) => void;
}

export const useDeviceStore = create<DeviceState>((set) => ({
  audioDevices: [],
  selectedDevice: null,
  
  setDevices: (devices) => set({ audioDevices: devices }),
  setSelectedDevice: (deviceId) => set({ selectedDevice: deviceId })
}));