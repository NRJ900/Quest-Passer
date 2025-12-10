export interface Game {
    id: string;
    name: string;
    icon?: string | null;
    splash?: string | null;
    executables?: Array<{
        os: string;
        name: string;
    }>;
}

export interface RunningGame {
    id: string;
    name: string;
    executable_name: string;
    pid?: number;
    startTime: number;
}

export interface Settings {
    notificationsEnabled: boolean;
    queueTimerDuration: number; // in seconds
}
