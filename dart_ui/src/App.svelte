<script>
    import { onMount } from "svelte";
    import { fly } from "svelte/transition";
    import { createDataProvider } from "./data/provider";

    let loading = true;
    let errorMessage = "";
    let payload = null;
    let gameState = null;
    let appVersion = "loading...";
    let provider = null;
    let unsubscribeProvider = null;
    let playersCount = 2;
    let startingScore = 501;

    const BOARD_SIZE = 500;
    const BOARD_CENTER = BOARD_SIZE / 2;
    const SINGLE_BULL_RADIUS = 15.9;
    const DOUBLE_BULL_RADIUS = 6.35;
    const TRIPLE_INNER_RADIUS = 97;
    const TRIPLE_OUTER_RADIUS = 105;
    const DOUBLE_INNER_RADIUS = 162;
    const DOUBLE_OUTER_RADIUS = 170;
    const BLACK_BORDER_RADIUS = 225;
    const SECTOR_ORDER = [
        20, 1, 18, 4, 13, 6, 10, 15, 2, 17, 3, 19, 7, 16, 8, 11, 14, 9, 12, 5,
    ];

    function cleanupProviderSubscription() {
        if (unsubscribeProvider) {
            unsubscribeProvider();
            unsubscribeProvider = null;
        }
    }

    function formatVersion(payload) {
        if (typeof payload === "string") {
            return payload;
        }

        if (payload && typeof payload === "object") {
            if (typeof payload.version === "string") {
                return payload.version;
            }

            if (typeof payload.app_version === "string") {
                return payload.app_version;
            }
        }

        return "unknown";
    }

    async function loadVersion() {
        if (!provider || typeof provider.getVersion !== "function") {
            appVersion = "n/a";
            return;
        }

        try {
            const versionPayload = await provider.getVersion();
            appVersion = formatVersion(versionPayload);
        } catch {
            appVersion = "unknown";
        }
    }

    async function initializeProvider() {
        cleanupProviderSubscription();

        const resolution = createDataProvider({
            serverBaseUrl: import.meta.env.VITE_SERVER_BASE_URL,
        });

        provider = resolution.provider;

        await loadVersion();

        unsubscribeProvider = await provider.subscribe({
            onScore: (nextPayload) => {
                payload = nextPayload;
                loading = false;
                errorMessage = "";
            },
            onError: (message) => {
                errorMessage = String(message);
            },
            onStatus: () => {},
            onGameState: (state) => {
                gameState = state;
                loading = false;
                errorMessage = "";
            },
            onGameOver: ({ winner_index, winner_name }) => {
                console.log(`🏆 ${winner_name} wins!`);
            },
        });

        // try to load the latest score from the server (e.g. after a page refresh)
        if (provider.getLatestScore) {
            try {
                const latest = await provider.getLatestScore();
                if (latest) {
                    payload = latest;
                    loading = false;
                    errorMessage = "";
                }
            } catch {
                // no score yet — will be populated by WS on first impact
            }
        }
    }

    async function startGame() {
        // placeholder — will launch the game view
        if (provider && typeof provider.startGame === "function") {
            try {
                await provider.startGame({ playersCount, startingScore });
            } catch (error) {
                console.error("Failed to start game:", error);
            }
        }
    }

    onMount(() => {
        void initializeProvider();

        return () => {
            cleanupProviderSubscription();
        };
    });

    function polarPoint(radius, angleDegrees) {
        const angleRadians = (angleDegrees * Math.PI) / 180;

        return {
            x: BOARD_CENTER + radius * Math.sin(angleRadians),
            y: BOARD_CENTER - radius * Math.cos(angleRadians),
        };
    }

    function ringSlicePath(innerRadius, outerRadius, startAngle, endAngle) {
        const outerStart = polarPoint(outerRadius, startAngle);
        const outerEnd = polarPoint(outerRadius, endAngle);
        const innerEnd = polarPoint(innerRadius, endAngle);
        const innerStart = polarPoint(innerRadius, startAngle);
        const largeArc = endAngle - startAngle > 180 ? 1 : 0;

        return [
            `M ${outerStart.x} ${outerStart.y}`,
            `A ${outerRadius} ${outerRadius} 0 ${largeArc} 1 ${outerEnd.x} ${outerEnd.y}`,
            `L ${innerEnd.x} ${innerEnd.y}`,
            `A ${innerRadius} ${innerRadius} 0 ${largeArc} 0 ${innerStart.x} ${innerStart.y}`,
            "Z",
        ].join(" ");
    }

    let showScorePopup = false;
    let scorePopupValue = 0;
    let scoreTimer;

    // Show a brief score popup whenever a new payload arrives, then auto-hide after 1.5s.
    $: if (payload) {
        scorePopupValue = payload.score;
        showScorePopup = true;
        clearTimeout(scoreTimer);
        scoreTimer = setTimeout(() => {
            showScorePopup = false;
        }, 1500);
    }

    $: displayImpactX = payload ? payload.impact_x : BOARD_CENTER;
    $: displayImpactY = payload ? BOARD_SIZE - payload.impact_y : BOARD_CENTER;
</script>

<svelte:head>
    <title>DartDetect — 501</title>
</svelte:head>

<main class="page">
    <section class="panel">
        <p class="version-badge">Version: {appVersion}</p>
        <div class="controls">
            <p class="eyebrow">DartDetect</p>

            <div class="hero">
                <h1>501</h1>
                <p class="lede">
                    The classic darts game. Throw three darts per turn and be
                    the first to reach zero.
                </p>
            </div>

            <div class="game-setup">
                <label class="field">
                    <span>Number of players</span>
                    <select bind:value={playersCount}>
                        <option value={1}>1 player</option>
                        <option value={2}>2 players</option>
                        <option value={3}>3 players</option>
                        <option value={4}>4 players</option>
                    </select>
                </label>

                <label class="field">
                    <span>Starting score</span>
                    <select bind:value={startingScore}>
                        <option value={301}>301</option>
                        <option value={501}>501</option>
                        <option value={701}>701</option>
                        <option value={1001}>1001</option>
                    </select>
                </label>

                <button class="start" on:click={startGame}>
                    Start the game
                </button>
            </div>

            {#if errorMessage && !loading}
                <p class="status error">{errorMessage}</p>
            {/if}
        </div>

        <div class="visuals">
            <figure class="board-panel">
                <svg
                    aria-label={payload
                        ? `Score ${payload.score}`
                        : "Dartboard"}
                    class="dartboard"
                    role="img"
                    viewBox="0 0 500 500"
                >
                    <circle
                        class="board-shadow"
                        cx={BOARD_CENTER}
                        cy={BOARD_CENTER}
                        r={BLACK_BORDER_RADIUS + 12}
                    />
                    <circle
                        class="board-rim"
                        cx={BOARD_CENTER}
                        cy={BOARD_CENTER}
                        r={BLACK_BORDER_RADIUS}
                    />

                    {#each SECTOR_ORDER as sector, index}
                        {@const centerAngle = index * 18}
                        {@const startAngle = centerAngle - 9}
                        {@const endAngle = centerAngle + 9}
                        {@const ringClass =
                            index % 2 === 0 ? "ring-red" : "ring-green"}
                        {@const singleClass =
                            index % 2 === 0 ? "single-light" : "single-dark"}
                        {@const dividerEnd = polarPoint(
                            BLACK_BORDER_RADIUS,
                            centerAngle + 9,
                        )}
                        {@const labelPoint = polarPoint(195, centerAngle)}

                        <path
                            class={singleClass}
                            d={ringSlicePath(
                                SINGLE_BULL_RADIUS,
                                DOUBLE_INNER_RADIUS,
                                startAngle,
                                endAngle,
                            )}
                        />
                        <path
                            class={ringClass}
                            d={ringSlicePath(
                                TRIPLE_INNER_RADIUS,
                                TRIPLE_OUTER_RADIUS,
                                startAngle,
                                endAngle,
                            )}
                        />
                        <path
                            class={ringClass}
                            d={ringSlicePath(
                                DOUBLE_INNER_RADIUS,
                                DOUBLE_OUTER_RADIUS,
                                startAngle,
                                endAngle,
                            )}
                        />
                        <line
                            class="sector-divider"
                            x1={BOARD_CENTER}
                            y1={BOARD_CENTER}
                            x2={dividerEnd.x}
                            y2={dividerEnd.y}
                        />
                        <text
                            class="sector-label"
                            x={labelPoint.x}
                            y={labelPoint.y}>{sector}</text
                        >
                    {/each}

                    <circle
                        class="bull-outer"
                        cx={BOARD_CENTER}
                        cy={BOARD_CENTER}
                        r={SINGLE_BULL_RADIUS}
                    />
                    <circle
                        class="bull-inner"
                        cx={BOARD_CENTER}
                        cy={BOARD_CENTER}
                        r={DOUBLE_BULL_RADIUS}
                    />

                    {#if payload}
                        <g>
                            <circle
                                class="impact-ring"
                                cx={displayImpactX}
                                cy={displayImpactY}
                                r="8"
                            />
                            <circle
                                class="impact-core"
                                cx={displayImpactX}
                                cy={displayImpactY}
                                r="4"
                            />
                        </g>
                    {/if}
                </svg>

                {#if showScorePopup}
                    <div
                        class="score-popup"
                        transition:fly={{ y: -12, duration: 350, opacity: 0 }}
                    >
                        +{scorePopupValue}
                    </div>
                {/if}
            </figure>

            {#if gameState}
                <div class="player-scores">
                    {#each gameState.players as player, i}
                        <div
                            class="player-score {i === gameState.current_player
                                ? 'active'
                                : ''}"
                        >
                            <span class="player-name">{player.name}</span>
                            <span class="player-remaining">{player.score}</span>
                        </div>
                    {/each}
                </div>
            {:else if payload}
                <p class="live-score">
                    Score: <strong>{payload.score}</strong>
                </p>
            {/if}
        </div>
    </section>
</main>
