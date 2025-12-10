import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Game, RunningGame, Settings } from "./types";
import { Search, Play, Square, Loader2, Settings as SettingsIcon, X, Plus, Trash2 } from "lucide-react";


function App() {
    const [games, setGames] = useState<Game[]>([]);
    const [search, setSearch] = useState("");
    const [loading, setLoading] = useState(true);
    const [runningGame, setRunningGame] = useState<RunningGame | null>(null);
    const [settings, setSettings] = useState<Settings>(() => {
        const saved = localStorage.getItem("qp-settings");
        return saved ? JSON.parse(saved) : { notificationsEnabled: false, queueTimerDuration: 930 };
    });
    const [showSettings, setShowSettings] = useState(false);

    // Quest Queue
    const [queue, setQueue] = useState<Game[]>([]);
    const [isQueueRunning, setIsQueueRunning] = useState(false);
    const [isStarting, setIsStarting] = useState(false);


    // Custom Games
    const [customGames, setCustomGames] = useState<Game[]>(() => {
        const saved = localStorage.getItem("qp-custom-games");
        return saved ? JSON.parse(saved) : [];
    });
    const [showAddGame, setShowAddGame] = useState(false);

    useEffect(() => {
        localStorage.setItem("qp-settings", JSON.stringify(settings));
    }, [settings]);

    useEffect(() => {
        localStorage.setItem("qp-custom-games", JSON.stringify(customGames));
    }, [customGames]);

    useEffect(() => {
        // Listen for process exit from backend
        const unlisten = listen("game_exited", () => {
            console.log("Game exited event received");
            setRunningGame(null);
        });

        // Load games
        async function loadGames() {
            try {
                const json = await invoke<string>("fetch_game_list");
                const data = JSON.parse(json);
                setGames(data);
            } catch (e) {
                console.error("Failed to load games", e);
            } finally {
                setLoading(false);
            }
        }
        loadGames();

        return () => {
            unlisten.then(f => f());
        };
        return () => {
            unlisten.then(f => f());
        };
    }, []);

    // Queue Logic: Auto-play next game when runningGame becomes null
    // Queue Logic: Auto-play next game when runningGame becomes null
    useEffect(() => {
        // Only run if queue is enabled, nothing is running, and we aren't already starting a game
        if (isQueueRunning && !runningGame && !isStarting) {
            if (queue.length > 0) {
                const nextGame = queue[0];
                console.log("Auto-starting next queued game:", nextGame.name);

                // Immediately update queue to prevent picking same game twice
                // And set isStarting to block effect from re-running
                setQueue(currentQueue => currentQueue.slice(1));

                // Trigger start
                startGame(nextGame);
            } else {
                // Queue finished
                console.log("Queue finished");
                setIsQueueRunning(false);
            }
        }
    }, [runningGame, isQueueRunning, queue, isStarting]);

    // Keyboard Shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === '/' && document.activeElement?.tagName !== 'INPUT') {
                e.preventDefault();
                const input = document.querySelector('input[type="text"]') as HTMLInputElement;
                if (input) input.focus();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, []);

    const stopGame = useCallback(async () => {
        if (!runningGame) return;
        console.log("Stopping game:", runningGame.name);
        try {
            await invoke("stop_process", { exec_name: runningGame.executable_name });
            setRunningGame(null);
        } catch (e) {
            console.error("Failed to stop game", e);
            alert(`Failed to stop game: ${e}`);
        }
    }, [runningGame]);

    // Queue Supervisor: Auto-stop game after 15m 30s if queue is running
    useEffect(() => {
        if (!runningGame || !isQueueRunning) return;

        const interval = setInterval(() => {
            const elapsedSeconds = Math.floor((Date.now() - runningGame.startTime) / 1000);

            // Dynamic duration from settings (default 15m 30s)
            const limit = settings.queueTimerDuration || 930;
            if (elapsedSeconds >= limit) {
                console.log("Queue Supervisor: Time limit reached, stopping game to proceed to next.");
                stopGame(); // This will trigger the 'game_exited' listener -> runningGame=null -> Queue Logic picks next
            }
        }, 5000); // Check every 5s

        return () => clearInterval(interval);
    }, [runningGame, isQueueRunning, settings.queueTimerDuration, stopGame]);

    // Derived filtered games
    const filteredGames = search.trim() === ""
        ? []
        : [...customGames, ...games].filter((g) => g.name.toLowerCase().includes(search.toLowerCase()));

    const [selectedGame, setSelectedGame] = useState<Game | null>(null);
    const [customGameName, setCustomGameName] = useState("");

    const handlePlayClick = (game: Game) => {
        setSelectedGame(game);
        setCustomGameName(game.name);
    };

    const handleDeleteGame = (e: React.MouseEvent, gameId: string) => {
        e.stopPropagation();
        if (confirm("Are you sure you want to delete this custom game?")) {
            setCustomGames(prev => prev.filter(g => g.id !== gameId));
        }
    };

    const handleConfirmStart = () => {
        if (selectedGame) {
            startGame(selectedGame, customGameName);
            setSelectedGame(null);
        }
    };

    const startGame = async (game: Game, nameOverride?: string) => {
        if (runningGame || isStarting) return;
        setIsStarting(true); // Lock

        try {
            const finalName = nameOverride || game.name;
            // Find executable name for win32
            const execName = game.executables?.find(e => e.os === 'win32')?.name || `${game.name}.exe`;
            // Keep the sanitized executable name based on original game to ensure it runs
            // Allow / and \ for relative paths (e.g. win64/game.exe), strip other invalid chars
            const sanitizedName = execName.replace(/[:*?"<>|]/g, "");

            await invoke("create_dummy_game", {
                path: "bin",
                executable_name: sanitizedName,
                app_id: game.id
            });

            const iconUrl = game.icon
                ? `https://cdn.discordapp.com/app-icons/${game.id}/${game.icon}.png?size=64`
                : null;

            await invoke("start_game_process", {
                name: finalName, // Use the custom name here
                path: "bin",
                executable_name: sanitizedName,
                app_id: game.id,
                icon_url: iconUrl
            });

            setRunningGame({
                id: game.id,
                name: finalName,
                executable_name: sanitizedName,
                startTime: Date.now()
            });

        } catch (e) {
            console.error("Failed to start game", e);
            alert(`Failed to start game: ${e} `);
        } finally {
            setIsStarting(false); // Unlock
        }
    };



    function ElapsedTime({ startTime }: { startTime: number }) {
        const [elapsed, setElapsed] = useState(0);
        const [notified, setNotified] = useState(false);

        useEffect(() => {
            const interval = setInterval(() => {
                const now = Date.now();
                const diff = Math.floor((now - startTime) / 1000);
                setElapsed(diff);

                // Notification check: 15m 30s = 930s
                if (settings.notificationsEnabled && diff >= 930 && !notified) {
                    playNotificationSound();
                    setNotified(true);
                }
            }, 1000);
            return () => clearInterval(interval);
        }, [startTime, notified, settings.notificationsEnabled]);

        const minutes = Math.floor(elapsed / 60);
        const seconds = elapsed % 60;
        return <span>{minutes}m {seconds}s</span>;
    }

    const playNotificationSound = () => {
        // Create a simple beep using Web Audio API or play a data URI sound
        // Using a simple short pleasant beep via Data URI for simplicity as no assets are guaranteed
        const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
        const osc = ctx.createOscillator();
        const gain = ctx.createGain();

        osc.connect(gain);
        gain.connect(ctx.destination);

        osc.type = "sine";
        osc.frequency.setValueAtTime(523.25, ctx.currentTime); // C5
        osc.frequency.exponentialRampToValueAtTime(1046.5, ctx.currentTime + 0.1); // C6

        gain.gain.setValueAtTime(0.1, ctx.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.5);

        osc.start();
        osc.stop(ctx.currentTime + 0.5);
    };

    return (
        <div className="min-h-screen bg-background text-foreground p-8 font-sans selection:bg-primary selection:text-primary-foreground relative overflow-x-hidden">
            <div className="mesh-bg" />
            <div className="max-w-4xl mx-auto space-y-8 relative z-10">

                {/* Header with Glassmorphism */}
                <div className="sticky top-0 z-40 -mx-8 px-8 py-4 backdrop-blur-md bg-background/30 border-b border-white/5 transition-all">
                    <div className="flex flex-col items-center space-y-4 text-center relative max-w-4xl mx-auto">
                        <div className="absolute right-0 top-0 flex gap-2">
                            {/* ... existing buttons ... */}
                            <button
                                className="p-2 text-muted-foreground hover:text-foreground transition-colors"
                                onClick={() => setShowAddGame(true)}
                                title="Add Custom Game"
                            >
                                <Plus className="w-6 h-6" />
                            </button>

                            <button
                                className="p-2 text-muted-foreground hover:text-foreground transition-colors"
                                onClick={() => setShowSettings(true)}
                                title="Settings"
                            >
                                <SettingsIcon className="w-6 h-6" />
                            </button>
                        </div>
                        <h1 className="text-4xl font-extrabold tracking-tight lg:text-5xl bg-gradient-to-r from-blue-400 to-purple-600 text-transparent bg-clip-text drop-shadow-sm">
                            Quest Passer
                        </h1>
                        <p className="text-muted-foreground text-lg max-w-2xl drop-shadow-sm transition-all duration-300 h-auto opacity-100">
                            Simulate Discord games to complete quests without installing them.
                        </p>
                    </div>
                </div>


                {/* Settings Modal */}
                {showSettings && (
                    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm animate-in fade-in">
                        <div className="w-full max-w-sm p-6 space-y-4 border bg-card rounded-lg shadow-lg animate-in zoom-in-95 relative">
                            <button
                                onClick={() => setShowSettings(false)}
                                className="absolute right-4 top-4 text-muted-foreground hover:text-foreground"
                            >
                                <X className="w-4 h-4" />
                            </button>
                            <div className="space-y-2">
                                <h3 className="text-lg font-semibold flex items-center gap-2">
                                    <SettingsIcon className="w-5 h-5" /> Settings
                                </h3>
                                <p className="text-sm text-muted-foreground">
                                    Configure application preferences.
                                </p>
                            </div>

                            <div className="space-y-4 pt-2">
                                <div className="space-y-4">
                                    <div className="flex items-center justify-between space-x-2">
                                        <div className="flex flex-col gap-1">
                                            <span className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                                Queue Timer Duration
                                            </span>
                                            <span className="text-xs text-muted-foreground">
                                                Run each game for {Math.floor((settings.queueTimerDuration || 930) / 60)}m {(settings.queueTimerDuration || 930) % 60}s
                                            </span>
                                        </div>
                                    </div>
                                    <input
                                        type="range"
                                        min="60"
                                        max="3600"
                                        step="10"
                                        value={settings.queueTimerDuration || 930}
                                        onChange={(e) => setSettings(s => ({ ...s, queueTimerDuration: parseInt(e.target.value) }))}
                                        className="w-full"
                                    />
                                    <p className="text-xs text-muted-foreground italic">
                                        Default is 15 minutes 30 seconds (930s). Minimum 1 minute.
                                    </p>
                                </div>

                                <div className="flex items-center justify-between space-x-2">
                                    <div className="flex flex-col gap-1">
                                        <span className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                            Completion Notification
                                        </span>
                                        <span className="text-xs text-muted-foreground">
                                            Play sound after 15m 30s
                                        </span>
                                    </div>
                                    <button
                                        role="switch"
                                        aria-checked={settings.notificationsEnabled}
                                        onClick={() => setSettings(s => ({ ...s, notificationsEnabled: !s.notificationsEnabled }))}
                                        className={`peer inline-flex h-[24px] w-[44px] shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 ${settings.notificationsEnabled ? "bg-primary" : "bg-input"}`}
                                    >
                                        <span
                                            data-state={settings.notificationsEnabled ? "checked" : "unchecked"}
                                            className={`pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform ${settings.notificationsEnabled ? "translate-x-5" : "translate-x-0"}`}
                                        />
                                    </button>
                                </div>
                            </div>

                            <div className="flex justify-end pt-4">
                                <button
                                    onClick={() => setShowSettings(false)}
                                    className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground shadow hover:bg-primary/90 h-9 px-4 py-2"
                                >
                                    Done
                                </button>
                            </div>
                        </div>
                    </div>
                )}

                {/* Add Game Modal */}
                {showAddGame && (
                    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm animate-in fade-in">
                        <div className="w-full max-w-md p-6 space-y-4 border bg-card rounded-lg shadow-lg animate-in zoom-in-95 relative">
                            <button
                                onClick={() => setShowAddGame(false)}
                                className="absolute right-4 top-4 text-muted-foreground hover:text-foreground"
                            >
                                <X className="w-4 h-4" />
                            </button>
                            <h3 className="text-lg font-semibold">Add Custom Game</h3>
                            <form onSubmit={(e) => {
                                e.preventDefault();
                                const formData = new FormData(e.currentTarget);
                                const name = formData.get("name") as string;
                                const id = formData.get("id") as string;
                                if (name && id) {
                                    // Try to find existing metadata (icon, etc) from known games
                                    const existing = games.find(g => g.id === id);
                                    const newGame: Game = {
                                        id,
                                        name, // Use user provided name or existing.name? User provided takes precedence here but we keep icon
                                        icon: existing?.icon,
                                        splash: existing?.splash,
                                        executables: existing?.executables || []
                                    };
                                    setCustomGames(prev => [...prev, newGame]);
                                    setShowAddGame(false);
                                }
                            }} className="space-y-4">
                                <div className="space-y-2">
                                    <label className="text-sm font-medium">Game Name</label>
                                    <input name="name" required className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="e.g. PUBG" />
                                </div>
                                <div className="space-y-2">
                                    <label className="text-sm font-medium">Application ID</label>
                                    <input name="id" required className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="e.g. 530196282960642048" />
                                    <p className="text-xs text-muted-foreground">Found in Discord Developer Portal or online databases.</p>
                                </div>
                                <button type="submit" className="w-full inline-flex items-center justify-center rounded-md bg-primary text-primary-foreground h-10 text-sm font-medium">Add Game</button>
                            </form>
                        </div>
                    </div>
                )}

                {/* Active Game Status & Queue Info */}
                {(runningGame || queue.length > 0) && (
                    <div className="space-y-4">
                        {runningGame && (
                            <div className="bg-card/50 backdrop-blur-md border border-white/10 rounded-xl p-6 shadow-2xl animate-in fade-in slide-in-from-top-4 ring-1 ring-primary/20 relative overflow-hidden">
                                <div className="absolute inset-0 bg-primary/5 pointer-events-none" />
                                <div className="flex items-center justify-between relative z-10">
                                    <div className="space-y-1">
                                        {/* ... existing status content ... */}
                                        <h3 className="text-lg font-semibold flex items-center gap-2">
                                            <span className="relative flex h-3 w-3">
                                                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                                                <span className="relative inline-flex rounded-full h-3 w-3 bg-green-500"></span>
                                            </span>
                                            Playing: {runningGame.name}
                                        </h3>
                                        <p className="text-sm text-muted-foreground">
                                            Started {new Date(runningGame.startTime).toLocaleTimeString()}
                                            <span className="mx-2">•</span>
                                            Duration: <ElapsedTime startTime={runningGame.startTime} />
                                        </p>
                                    </div>
                                    <button
                                        onClick={stopGame}
                                        className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 bg-destructive text-destructive-foreground shadow hover:bg-destructive/90 h-9 px-4 py-2 gap-2"
                                    >
                                        <Square className="h-4 w-4" /> Stop
                                    </button>
                                </div>
                            </div>
                        )}

                        {queue.length > 0 && (
                            <div className="bg-card/30 backdrop-blur border border-white/5 rounded-lg p-4">
                                <div className="flex items-center justify-between mb-2">
                                    <h4 className="text-sm font-semibold flex items-center gap-2">
                                        Quest Queue ({queue.length})
                                        {isQueueRunning ? (
                                            <span className="text-xs bg-green-500/20 text-green-400 px-2 py-0.5 rounded-full">Active</span>
                                        ) : (
                                            <span className="text-xs bg-yellow-500/20 text-yellow-400 px-2 py-0.5 rounded-full">Paused</span>
                                        )}
                                        {/* Queue Estimator */}
                                        {queue.length > 0 && (
                                            <span className="text-xs text-muted-foreground ml-2">
                                                Est. Finish: {new Date(Date.now() + (queue.length * (settings.queueTimerDuration || 930) * 1000) + (runningGame ? ((settings.queueTimerDuration || 930) * 1000) - (Date.now() - runningGame.startTime) : 0)).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                                            </span>
                                        )}
                                    </h4>
                                    <div className="flex gap-2">
                                        {!isQueueRunning && !runningGame && (
                                            <button
                                                onClick={() => {
                                                    // Immediately trigger start mechanism via state
                                                    if (queue.length > 0) {
                                                        setIsQueueRunning(true);
                                                        // Effect will pick it up
                                                    }
                                                }}
                                                className="text-xs bg-primary/20 text-primary hover:bg-primary/30 px-3 py-1 rounded-md transition-colors"
                                            >
                                                Start Queue
                                            </button>
                                        )}
                                        {isQueueRunning && (
                                            <button
                                                onClick={() => setIsQueueRunning(false)}
                                                className="text-xs bg-yellow-500/10 text-yellow-500 hover:bg-yellow-500/20 px-3 py-1 rounded-md transition-colors"
                                            >
                                                Pause
                                            </button>
                                        )}
                                        <button
                                            onClick={() => { setQueue([]); setIsQueueRunning(false); }}
                                            className="text-xs text-muted-foreground hover:text-destructive px-2"
                                        >
                                            Clear
                                        </button>
                                    </div>
                                </div>
                                <div className="space-y-1">
                                    {queue.map((g, i) => (
                                        <div key={i} className="flex items-center justify-between text-sm p-2 bg-black/20 rounded">
                                            <span>{i + 1}. {g.name}</span>
                                            <button onClick={() => setQueue(q => q.filter((_, idx) => idx !== i))} className="text-muted-foreground hover:text-white"><X className="w-3 h-3" /></button>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}
                    </div>
                )}

                {/* Start Game Modal */}
                {selectedGame && (
                    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm animate-in fade-in">
                        <div className="w-full max-w-md p-6 space-y-4 border bg-card rounded-lg shadow-lg animate-in zoom-in-95">
                            <div className="space-y-2">
                                <h3 className="text-lg font-semibold">Start Game</h3>
                                <p className="text-sm text-muted-foreground">
                                    You can edit the game name before starting. This name will appear in the runner window.
                                </p>
                            </div>
                            <div className="space-y-2">
                                <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                    Game Name
                                </label>
                                <input
                                    className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                                    value={customGameName}
                                    onChange={(e) => setCustomGameName(e.target.value)}
                                    // Add Enter key support
                                    onKeyDown={(e) => e.key === 'Enter' && handleConfirmStart()}
                                    autoFocus
                                />
                            </div>
                            <div className="flex justify-end gap-3">
                                <button
                                    onClick={() => setSelectedGame(null)}
                                    className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-input bg-transparent shadow-sm hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
                                >
                                    Cancel
                                </button>
                                <button
                                    onClick={handleConfirmStart}
                                    className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground shadow hover:bg-primary/90 h-9 px-4 py-2"
                                >
                                    Start Game
                                </button>
                            </div>
                        </div>
                    </div>
                )}

                {/* Search */}
                <div className="relative">
                    <Search className="absolute left-3 top-3 h-5 w-5 text-muted-foreground" />
                    <input
                        type="text"
                        placeholder="Search for a game to get started..."
                        className="flex h-12 w-full rounded-md border border-input bg-background px-3 py-2 pl-10 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                    />
                </div>

                {/* Game List Area */}
                <div className="min-h-[300px]">
                    {search.trim() === "" ? (
                        // Instructions View
                        <div className="flex flex-col items-center justify-center space-y-4 py-20 text-center text-muted-foreground">
                            <Search className="h-16 w-16 opacity-20" />
                            <div className="space-y-2">
                                <h3 className="text-lg font-semibold text-foreground">Ready to Play?</h3>
                                <p>Search for a game above to see available quests.</p>
                                <p className="text-sm">Can't find it? Click the <strong>+</strong> button to add a custom game ID.</p>
                            </div>
                        </div>
                    ) : (
                        // Results View
                        loading ? (
                            <div className="flex justify-center py-20">
                                <Loader2 className="h-10 w-10 animate-spin text-primary" />
                            </div>
                        ) : (
                            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                {filteredGames.slice(0, 50).map((game) => (
                                    <div
                                        key={game.id}
                                        className="group relative flex flex-col justify-between space-y-2 rounded-lg border border-border bg-card p-4 hover:bg-accent/50 transition-colors"
                                    >
                                        <div className="space-y-2">
                                            <div className="flex items-center gap-3">
                                                {game.icon ? (
                                                    <img
                                                        src={`https://cdn.discordapp.com/app-icons/${game.id}/${game.icon}.png?size=64`}
                                                        alt={game.name}
                                                        className="w-10 h-10 rounded-md object-cover"
                                                        onError={(e) => {
                                                            (e.target as HTMLImageElement).style.display = 'none';
                                                        }}
                                                    />
                                                ) : (
                                                    <div className="w-10 h-10 rounded-md bg-muted flex items-center justify-center">
                                                        <Search className="w-5 h-5 text-muted-foreground opacity-50" />
                                                    </div>
                                                )}
                                                <div className="flex-1 min-w-0">
                                                    <div className="flex items-center justify-between">
                                                        <h3 className="font-semibold tracking-tight truncate" title={game.name}>
                                                            {game.name}
                                                        </h3>
                                                        {/* Delete Button for Custom Games */}
                                                        {customGames.some(cg => cg.id === game.id) && (
                                                            <button
                                                                onClick={(e) => handleDeleteGame(e, game.id)}
                                                                className="text-muted-foreground hover:text-destructive transition-colors p-1"
                                                                title="Delete Custom Game"
                                                            >
                                                                <Trash2 className="w-4 h-4" />
                                                            </button>
                                                        )}
                                                    </div>
                                                    <p className="text-xs text-muted-foreground truncate">
                                                        ID: {game.id}
                                                    </p>
                                                </div>
                                            </div>
                                        </div>
                                        <button
                                            onClick={() => handlePlayClick(game)}
                                            disabled={!!runningGame}
                                            className="w-full inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground shadow hover:bg-primary/90 h-9 px-4 py-2 gap-2 mt-2"
                                        >
                                            <Play className="h-4 w-4" /> Play
                                        </button>
                                        {(() => {
                                            const isRunning = runningGame?.id === game.id;
                                            const isInQueue = queue.some(g => g.id === game.id);

                                            if (isRunning) {
                                                return (
                                                    <button disabled className="w-full inline-flex items-center justify-center rounded-md text-sm font-medium border border-transparent bg-green-500/20 text-green-500 h-9 px-4 py-2 mt-2 gap-2 cursor-not-allowed">
                                                        <Play className="h-4 w-4" /> Playing...
                                                    </button>
                                                );
                                            }

                                            if (isInQueue) {
                                                return (
                                                    <button disabled className="w-full inline-flex items-center justify-center rounded-md text-sm font-medium border border-transparent bg-yellow-500/20 text-yellow-500 h-9 px-4 py-2 mt-2 gap-2 cursor-not-allowed">
                                                        <Loader2 className="h-4 w-4 animate-spin" /> In Queue
                                                    </button>
                                                );
                                            }

                                            return (
                                                <button
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        setQueue(q => {
                                                            if (q.some(existing => existing.id === game.id)) return q;
                                                            return [...q, game];
                                                        });
                                                        // Auto-enable queue if nothing is running so it starts immediately
                                                        if (!runningGame && !isQueueRunning) {
                                                            setIsQueueRunning(true);
                                                        }
                                                    }}
                                                    className="w-full inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-input bg-transparent hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2 mt-2 gap-2"
                                                >
                                                    <Plus className="h-4 w-4" /> Queue
                                                </button>
                                            );
                                        })()}
                                    </div>
                                ))}
                                {filteredGames.length === 0 && (
                                    <div className="col-span-full text-center py-10 text-muted-foreground">
                                        No games found matching "{search}".
                                    </div>
                                )}
                            </div>
                        )
                    )}
                </div>

            </div>
        </div>
    );
}

export default App;
