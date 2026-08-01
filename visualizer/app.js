// State management
let state = {
    profiles: {}, // profileId -> { id, name, sessions }
    activeProfileId: null,
    filters: {
        time: 'all',
        level: 'all',
        mode: 'all'
    },
    heatmapMode: 'finger' // 'finger' | 'key-errors' | 'load'
};

// Chart instances
let wpmChartInstance = null;
let fingerLoadChartInstance = null;
let levelPerfChartInstance = null;

// QWERTY Key to Finger mapping
const FINGER_MAPPING = {
    'left_pinky': ['`', '1', 'q', 'a', 'z'],
    'left_ring': ['2', 'w', 's', 'x'],
    'left_middle': ['3', 'e', 'd', 'c'],
    'left_index': ['4', '5', 'r', 't', 'f', 'g', 'v', 'b'],
    'left_thumb': [],
    
    'right_thumb': [' '],
    'right_index': ['6', '7', 'y', 'u', 'h', 'j', 'n', 'm'],
    'right_middle': ['8', 'i', 'k', ','],
    'right_ring': ['9', 'o', 'l', '.'],
    'right_pinky': ['0', '-', '=', 'p', '[', ']', '\\', ';', "'", '/']
};

// Reverse map for key lookup
const KEY_TO_FINGER = {};
Object.entries(FINGER_MAPPING).forEach(([finger, keys]) => {
    keys.forEach(key => {
        KEY_TO_FINGER[key.toLowerCase()] = finger;
    });
});

const FINGER_DISPLAY_NAMES = {
    'left_pinky': 'Left Pinky',
    'left_ring': 'Left Ring',
    'left_middle': 'Left Middle',
    'left_index': 'Left Index',
    'left_thumb': 'Left Thumb',
    'right_thumb': 'Right Thumb',
    'right_index': 'Right Index',
    'right_middle': 'Right Middle',
    'right_ring': 'Right Ring',
    'right_pinky': 'Right Pinky'
};

// DOM Elements
const dropZone = document.getElementById('drop-zone');
const fileInput = document.getElementById('csv-file-input');
const profileList = document.getElementById('profile-list');
const emptyDashboard = document.getElementById('empty-dashboard');
const dashboardGrid = document.getElementById('dashboard-grid');

const activeProfileName = document.getElementById('active-profile-name');
const activeProfileSubtitle = document.getElementById('active-profile-subtitle');
const renameProfileBtn = document.getElementById('rename-profile-btn');
const deleteProfileBtn = document.getElementById('delete-profile-btn');

// Filters
const filterTime = document.getElementById('filter-time');
const filterLevel = document.getElementById('filter-level');
const filterMode = document.getElementById('filter-mode');
const filterSummary = document.getElementById('filter-summary');

// Stats elements
const statAvgWpm = document.getElementById('stat-avg-wpm');
const statPeakWpm = document.getElementById('stat-peak-wpm');
const statAvgAccuracy = document.getElementById('stat-avg-accuracy');
const statTotalKeystrokes = document.getElementById('stat-total-keystrokes');
const statErrorCount = document.getElementById('stat-error-count');
const statStreak = document.getElementById('stat-streak');

// Heatmap mode buttons
const btnModeFinger = document.getElementById('btn-mode-finger');
const btnModeKeyErrors = document.getElementById('btn-mode-key-errors');
const btnModeLoad = document.getElementById('btn-mode-load');

// Table element
const historyTbody = document.getElementById('history-tbody');

// Modal Elements
const modalOverlay = document.getElementById('key-modal-overlay');
const modalCloseBtn = document.getElementById('modal-close-btn');
const modalKeyDisplay = document.getElementById('modal-key-display');
const modalKeyTitle = document.getElementById('modal-key-title');
const modalFingerName = document.getElementById('modal-finger-name');
const modalStatKeystrokes = document.getElementById('modal-stat-keystrokes');
const modalStatErrors = document.getElementById('modal-stat-errors');
const modalStatAccuracy = document.getElementById('modal-stat-accuracy');
const modalMistakesList = document.getElementById('modal-mistakes-list');

// Initialize App
document.addEventListener('DOMContentLoaded', () => {
    loadStateFromLocalStorage();
    setupEventListeners();
    renderProfileList();
    
    if (state.activeProfileId && state.profiles[state.activeProfileId]) {
        showDashboard(state.activeProfileId);
    } else {
        showEmptyState();
    }
});

// Setup Event Listeners
function setupEventListeners() {
    // Drag & Drop events
    ['dragenter', 'dragover'].forEach(eventName => {
        dropZone.addEventListener(eventName, (e) => {
            e.preventDefault();
            dropZone.classList.add('dragover');
        }, false);
    });

    ['dragleave', 'drop'].forEach(eventName => {
        dropZone.addEventListener(eventName, (e) => {
            e.preventDefault();
            dropZone.classList.remove('dragover');
        }, false);
    });

    dropZone.addEventListener('drop', (e) => {
        const dt = e.dataTransfer;
        const files = dt.files;
        if (files.length > 0) {
            handleCsvFile(files[0]);
        }
    });

    dropZone.addEventListener('click', () => {
        fileInput.click();
    });

    fileInput.addEventListener('change', (e) => {
        if (e.target.files.length > 0) {
            handleCsvFile(e.target.files[0]);
        }
    });

    // Profile actions
    renameProfileBtn.addEventListener('click', () => {
        if (!state.activeProfileId) return;
        const profile = state.profiles[state.activeProfileId];
        const newName = prompt('Enter new profile name:', profile.name);
        if (newName && newName.trim()) {
            profile.name = newName.trim();
            saveStateToLocalStorage();
            renderProfileList();
            activeProfileName.textContent = profile.name;
        }
    });

    deleteProfileBtn.addEventListener('click', () => {
        if (!state.activeProfileId) return;
        if (confirm('Are you sure you want to delete this profile?')) {
            delete state.profiles[state.activeProfileId];
            const remainingIds = Object.keys(state.profiles);
            state.activeProfileId = remainingIds.length > 0 ? remainingIds[0] : null;
            saveStateToLocalStorage();
            renderProfileList();
            
            if (state.activeProfileId) {
                showDashboard(state.activeProfileId);
            } else {
                showEmptyState();
            }
        }
    });

    const clearSessionsBtn = document.getElementById('clear-sessions-btn');
    if (clearSessionsBtn) {
        clearSessionsBtn.addEventListener('click', () => {
            if (!state.activeProfileId) return;
            const profile = state.profiles[state.activeProfileId];
            if (confirm(`Are you sure you want to clear all ${profile.sessions.length} recorded session(s) for "${profile.name}"?`)) {
                profile.sessions = [];
                saveStateToLocalStorage();
                renderProfileList();
                renderDashboardContent(state.activeProfileId);
                activeProfileSubtitle.textContent = 'Session history cleared (0 sessions recorded)';
            }
        });
    }

    const sampleCsvText = `timestamp,wpm,overall_accuracy,level,training_mode,duration_seconds,finger_accuracy_left_pinky,finger_accuracy_left_ring,finger_accuracy_left_middle,finger_accuracy_left_index,finger_accuracy_left_thumb,finger_accuracy_right_thumb,finger_accuracy_right_index,finger_accuracy_right_middle,finger_accuracy_right_ring,finger_accuracy_right_pinky,finger_keystrokes_left_pinky,finger_keystrokes_left_ring,finger_keystrokes_left_middle,finger_keystrokes_left_index,finger_keystrokes_left_thumb,finger_keystrokes_right_thumb,finger_keystrokes_right_index,finger_keystrokes_right_middle,finger_keystrokes_right_ring,finger_keystrokes_right_pinky,mistake_matrix
1785614000,48.5,92.4,Beginner,Words,45,88.2,94.1,96.5,91.0,100.0,100.0,90.5,95.0,91.2,85.0,18,22,35,42,0,15,38,29,21,12,"a:s:2|q:w:1|e:r:1|p:o:2"
1785614300,53.2,94.8,Intermediate,Paragraph,60,91.5,96.0,98.0,94.2,100.0,100.0,93.0,97.1,93.5,88.0,24,28,45,56,0,20,52,36,27,16,"s:a:1|w:e:1|i:o:1|n:m:1"
1785614600,59.8,96.5,Intermediate,Paragraph,60,94.0,98.2,99.0,96.0,100.0,100.0,95.5,98.5,95.0,91.5,30,32,52,68,0,25,60,42,31,20,"e:r:1|t:y:1|r:e:1"
1785614900,64.1,97.2,Advanced,Full,75,95.8,99.0,97.5,100.0,100.0,97.0,99.0,96.8,93.2,35,38,60,78,0,30,72,50,38,24,"p:o:1|a:s:1"
1785615200,68.4,98.6,Advanced,Full,90,97.0,99.5,100.0,98.8,100.0,100.0,98.2,99.5,98.0,95.0,40,42,68,88,0,35,82,58,44,28,"c:v:1"`;

    function loadDemoData() {
        handleCsvContent(sampleCsvText, 'Demo Profile (Sample Data)');
    }

    const loadDemoBtn = document.getElementById('load-demo-btn');
    const emptyDemoBtn = document.getElementById('empty-demo-btn');
    const emptyBrowseBtn = document.getElementById('empty-browse-btn');

    if (loadDemoBtn) loadDemoBtn.addEventListener('click', loadDemoData);
    if (emptyDemoBtn) emptyDemoBtn.addEventListener('click', loadDemoData);
    if (emptyBrowseBtn) emptyBrowseBtn.addEventListener('click', () => fileInput.click());

    // Filters
    [filterTime, filterLevel, filterMode].forEach(el => {
        el.addEventListener('change', () => {
            state.filters.time = filterTime.value;
            state.filters.level = filterLevel.value;
            state.filters.mode = filterMode.value;
            saveStateToLocalStorage();
            if (state.activeProfileId) {
                renderDashboardContent(state.activeProfileId);
            }
        });
    });

    // Heatmap mode toggle
    btnModeFinger.addEventListener('click', () => setHeatmapMode('finger'));
    btnModeKeyErrors.addEventListener('click', () => setHeatmapMode('key-errors'));
    btnModeLoad.addEventListener('click', () => setHeatmapMode('load'));

    // Keyboard click inspector
    document.querySelectorAll('.key').forEach(keyEl => {
        keyEl.addEventListener('click', () => {
            const keyValue = keyEl.getAttribute('data-key');
            if (keyValue !== null && state.activeProfileId) {
                openKeyModal(keyValue);
            }
        });
    });

    // Modal close
    modalCloseBtn.addEventListener('click', closeModal);
    modalOverlay.addEventListener('click', (e) => {
        if (e.target === modalOverlay) closeModal();
    });

    // Documentation Modal Handlers
    const openDocsBtn = document.getElementById('open-docs-btn');
    const docsModalOverlay = document.getElementById('docs-modal-overlay');
    const docsCloseBtn = document.getElementById('docs-close-btn');

    if (openDocsBtn && docsModalOverlay) {
        openDocsBtn.addEventListener('click', () => {
            docsModalOverlay.classList.remove('hidden');
        });
    }

    if (docsCloseBtn && docsModalOverlay) {
        docsCloseBtn.addEventListener('click', () => {
            docsModalOverlay.classList.add('hidden');
        });
    }

    if (docsModalOverlay) {
        docsModalOverlay.addEventListener('click', (e) => {
            if (e.target === docsModalOverlay) {
                docsModalOverlay.classList.add('hidden');
            }
        });
    }

    // Copy to Clipboard logic for code snippets
    document.querySelectorAll('.btn-copy').forEach(btn => {
        btn.addEventListener('click', () => {
            const targetId = btn.getAttribute('data-target');
            const codeEl = document.getElementById(targetId);
            if (codeEl) {
                const textToCopy = codeEl.textContent.trim();
                navigator.clipboard.writeText(textToCopy).then(() => {
                    const originalHtml = btn.innerHTML;
                    btn.classList.add('copied');
                    btn.innerHTML = `✓ Copied!`;
                    setTimeout(() => {
                        btn.classList.remove('copied');
                        btn.innerHTML = originalHtml;
                    }, 2000);
                }).catch(err => {
                    console.error('Failed to copy text: ', err);
                });
            }
        });
    });
}

function setHeatmapMode(mode) {
    state.heatmapMode = mode;
    [btnModeFinger, btnModeKeyErrors, btnModeLoad].forEach(b => b.classList.remove('active'));
    if (mode === 'finger') btnModeFinger.classList.add('active');
    else if (mode === 'key-errors') btnModeKeyErrors.classList.add('active');
    else if (mode === 'load') btnModeLoad.classList.add('active');

    if (state.activeProfileId) {
        const filtered = getFilteredSessions(state.profiles[state.activeProfileId].sessions);
        renderKeyboardHeatmap(filtered);
    }
}

// Handle CSV File Upload
function handleCsvFile(file) {
    if (!file.name.endsWith('.csv')) {
        alert('Please upload a valid CSV file.');
        return;
    }

    const reader = new FileReader();
    reader.onload = function (e) {
        const text = e.target.result;
        parseAndLoadCsv(text, file.name);
    };
    reader.readAsText(file);
}

// Optimized Parser for CSV
function parseAndLoadCsv(csvText, filename) {
    const rawLines = csvText.split(/\r?\n/);
    const lines = [];
    for (let i = 0; i < rawLines.length; i++) {
        const trimmed = rawLines[i].trim();
        if (trimmed.length > 0) lines.push(trimmed);
    }
    
    if (lines.length < 2) {
        alert('CSV file is empty or corrupted.');
        return;
    }

    const header = parseCsvLine(lines[0]);
    
    // Validate required headers
    if (!header.includes('timestamp') || !header.includes('wpm')) {
        alert('Invalid CSV file. Must be a FingerTrack exported CSV.');
        return;
    }

    const fingerKeys = [
        'left_pinky', 'left_ring', 'left_middle', 'left_index', 'left_thumb',
        'right_thumb', 'right_index', 'right_middle', 'right_ring', 'right_pinky'
    ];

    // Pre-build index-to-handler map ONCE for header columns
    const columnHandlers = header.map(rawCol => {
        const colName = rawCol.trim();
        if (colName === 'date_time') return (session, val) => session.date_time = val;
        if (colName === 'timestamp') return (session, val) => {
            session.timestamp = parseInt(val, 10) || 0;
            if (!session.date_time && session.timestamp > 0) {
                session.date_time = new Date(session.timestamp * 1000).toLocaleString();
            }
        };
        if (colName === 'session_duration_secs') return (session, val) => session.session_duration_secs = parseFloat(val) || null;
        if (colName === 'level') return (session, val) => session.level = val || 'Beginner';
        if (colName === 'training_mode') return (session, val) => session.training_mode = val || 'Random';
        if (colName === 'round_length') return (session, val) => session.round_length = val || 'Medium';
        if (colName === 'wpm') return (session, val) => session.wpm = parseFloat(val) || 0.0;
        if (colName === 'overall_accuracy') return (session, val) => session.overall_accuracy = val === '' ? null : parseFloat(val);
        if (colName === 'total_keystrokes') return (session, val) => session.total_keystrokes = val === '' ? null : parseInt(val, 10);
        if (colName === 'correct_keystrokes') return (session, val) => session.correct_keystrokes = val === '' ? null : parseInt(val, 10);
        if (colName === 'error_count') return (session, val) => session.error_count = val === '' ? null : parseInt(val, 10);
        if (colName === 'top_mistakes' || colName === 'mistake_matrix') return (session, val) => session.top_mistakes = val;

        if (fingerKeys.includes(colName)) {
            return (session, val) => session.finger_accuracies[colName] = val === '' ? null : parseFloat(val);
        }
        for (let i = 0; i < fingerKeys.length; i++) {
            const f = fingerKeys[i];
            if (colName === `${f}_count` || colName === `finger_keystrokes_${f}`) {
                return (session, val) => session.finger_keystrokes[f] = val === '' ? null : parseInt(val, 10);
            }
            if (colName === `${f}_errors` || colName === `finger_errors_${f}`) {
                return (session, val) => session.finger_errors[f] = val === '' ? null : parseInt(val, 10);
            }
            if (colName === `finger_accuracy_${f}`) {
                return (session, val) => session.finger_accuracies[f] = val === '' ? null : parseFloat(val);
            }
        }
        return null;
    });

    const sessions = new Array(lines.length - 1);
    let validCount = 0;

    for (let i = 1; i < lines.length; i++) {
        const fields = parseCsvLine(lines[i]);
        if (fields.length !== header.length) continue;

        const session = {
            date_time: '',
            timestamp: 0,
            session_duration_secs: null,
            level: 'Beginner',
            training_mode: 'Random',
            round_length: 'Medium',
            wpm: 0.0,
            overall_accuracy: null,
            total_keystrokes: null,
            correct_keystrokes: null,
            error_count: null,
            finger_accuracies: {},
            finger_keystrokes: {},
            finger_errors: {},
            top_mistakes: ''
        };

        for (let j = 0; j < columnHandlers.length; j++) {
            const handler = columnHandlers[j];
            if (handler) handler(session, fields[j]);
        }

        if (session.overall_accuracy === null) {
            let sumAcc = 0, countAcc = 0;
            const accs = Object.values(session.finger_accuracies);
            for (let k = 0; k < accs.length; k++) {
                if (accs[k] !== null) { sumAcc += accs[k]; countAcc++; }
            }
            session.overall_accuracy = countAcc > 0 ? sumAcc / countAcc : 100.0;
        }

        sessions[validCount++] = session;
    }

    sessions.length = validCount;

    if (sessions.length === 0) {
        alert('Could not parse any valid typing session rows.');
        return;
    }

    sessions.sort((a, b) => a.timestamp - b.timestamp);

    const firstTimestamp = sessions[0].timestamp;
    const profileId = `profile_${firstTimestamp}_${sessions.length}`;

    let baseName = filename.replace(/\.[^/.]+$/, "");
    baseName = baseName.replace(/_export$/, "");
    const profileName = baseName.charAt(0).toUpperCase() + baseName.slice(1);

    state.profiles[profileId] = {
        id: profileId,
        name: profileName,
        sessions: sessions
    };
    state.activeProfileId = profileId;

    saveStateToLocalStorage();
    renderProfileList();
    showDashboard(profileId);
}

    if (sessions.length === 0) {
        alert('Could not parse any valid typing session rows.');
        return;
    }

    sessions.sort((a, b) => a.timestamp - b.timestamp);

    const firstTimestamp = sessions[0].timestamp;
    const profileId = `profile_${firstTimestamp}_${sessions.length}`;

    let baseName = filename.replace(/\.[^/.]+$/, "");
    baseName = baseName.replace(/_export$/, "");
    baseName = baseName.charAt(0).toUpperCase() + baseName.slice(1);

    const profileName = `${baseName} (${sessions.length} sessions)`;

    state.profiles[profileId] = {
        id: profileId,
        name: profileName,
        sessions: sessions
    };
    state.activeProfileId = profileId;

    saveStateToLocalStorage();
    renderProfileList();
    showDashboard(profileId);
}

function parseCsvLine(line) {
    const result = [];
    let insideQuote = false;
    let entry = '';
    for (let i = 0; i < line.length; i++) {
        const char = line[i];
        if (char === '"') {
            insideQuote = !insideQuote;
        } else if (char === ',' && !insideQuote) {
            result.push(entry.trim());
            entry = '';
        } else {
            entry += char;
        }
    }
    result.push(entry.trim());
    return result;
}

// UI State Toggles
function showEmptyState() {
    emptyDashboard.classList.remove('hidden');
    dashboardGrid.classList.add('hidden');
}

function showDashboard(profileId) {
    emptyDashboard.classList.add('hidden');
    dashboardGrid.classList.remove('hidden');
    
    const profile = state.profiles[profileId];
    activeProfileName.textContent = profile.name;
    activeProfileSubtitle.textContent = `Loaded ${profile.sessions.length} practice session(s)`;
    
    renderDashboardContent(profileId);
}

// Filter Session Logic
function getFilteredSessions(sessions) {
    const nowSecs = Math.floor(Date.now() / 1000);
    return sessions.filter(s => {
        // Time filter
        if (state.filters.time === '7d' && (nowSecs - s.timestamp) > 7 * 86400) return false;
        if (state.filters.time === '30d' && (nowSecs - s.timestamp) > 30 * 86400) return false;

        // Level filter
        if (state.filters.level !== 'all' && s.level && s.level.toLowerCase() !== state.filters.level.toLowerCase()) return false;

        // Mode filter
        if (state.filters.mode !== 'all' && s.training_mode && !s.training_mode.toLowerCase().includes(state.filters.mode.toLowerCase())) return false;

        return true;
    });
}

function renderDashboardContent(profileId) {
    const profile = state.profiles[profileId];
    const filtered = getFilteredSessions(profile.sessions);

    filterSummary.textContent = `Showing ${filtered.length} of ${profile.sessions.length} session(s)`;

    calculateAndRenderStats(filtered);
    renderCharts(filtered);
    renderKeyboardHeatmap(filtered);
    renderHistoryTable(filtered);
}

// Calculations and Stats UI
function calculateAndRenderStats(sessions) {
    const total = sessions.length;

    let sumWpm = 0, peakWpm = 0;
    let sumAcc = 0;
    let totalStrokes = 0, totalErrors = 0;

    sessions.forEach(s => {
        sumWpm += s.wpm;
        if (s.wpm > peakWpm) peakWpm = s.wpm;
        sumAcc += s.overall_accuracy !== null ? s.overall_accuracy : 100.0;
        if (s.total_keystrokes !== null) totalStrokes += s.total_keystrokes;
        if (s.error_count !== null) totalErrors += s.error_count;
    });

    statAvgWpm.textContent = total > 0 ? (sumWpm / total).toFixed(1) : '0.0';
    statPeakWpm.textContent = peakWpm.toFixed(1);
    statAvgAccuracy.textContent = total > 0 ? `${(sumAcc / total).toFixed(1)}%` : '0.0%';
    statTotalKeystrokes.textContent = totalStrokes.toLocaleString();
    statErrorCount.textContent = `${totalErrors.toLocaleString()} total errors`;

    // Streak calculation
    let currentStreak = 0;
    sessions.forEach(s => {
        if (s.overall_accuracy >= 92.0) {
            currentStreak++;
        } else {
            currentStreak = 0;
        }
    });
    statStreak.textContent = currentStreak;
}

// Chart.js renderers
function renderCharts(sessions) {
    if (wpmChartInstance) wpmChartInstance.destroy();
    if (fingerLoadChartInstance) fingerLoadChartInstance.destroy();
    if (levelPerfChartInstance) levelPerfChartInstance.destroy();

    // Chart 1: WPM & Accuracy Trend Dual Axis
    const wpmLabels = sessions.map((s, idx) => `S${idx + 1}`);
    const wpmData = sessions.map(s => s.wpm);
    const accData = sessions.map(s => s.overall_accuracy !== null ? s.overall_accuracy : 100.0);
    const dateLabels = sessions.map(s => s.date_time);

    const wpmCtx = document.getElementById('wpmTrendChart').getContext('2d');
    wpmChartInstance = new Chart(wpmCtx, {
        type: 'line',
        data: {
            labels: wpmLabels,
            datasets: [
                {
                    label: 'Speed (WPM)',
                    data: wpmData,
                    borderColor: '#E0B4B2',
                    backgroundColor: 'rgba(224, 180, 178, 0.12)',
                    borderWidth: 2.5,
                    fill: true,
                    tension: 0.35,
                    yAxisID: 'y'
                },
                {
                    label: 'Accuracy (%)',
                    data: accData,
                    borderColor: '#52B788',
                    backgroundColor: 'transparent',
                    borderWidth: 2,
                    borderDash: [4, 4],
                    tension: 0.35,
                    yAxisID: 'y1'
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            animation: { duration: 150 },
            plugins: {
                legend: { display: true, labels: { color: '#ABAFB5', font: { family: 'Inter', size: 11 } } },
                tooltip: {
                    callbacks: {
                        title: (items) => dateLabels[items[0].dataIndex],
                        label: (item) => item.datasetIndex === 0 ? ` Speed: ${item.raw.toFixed(1)} WPM` : ` Accuracy: ${item.raw.toFixed(1)}%`
                    }
                }
            },
            scales: {
                x: { grid: { color: 'rgba(103, 126, 138, 0.15)' }, ticks: { color: '#ABAFB5' } },
                y: { type: 'linear', display: true, position: 'left', title: { display: true, text: 'WPM', color: '#E0B4B2' }, grid: { color: 'rgba(103, 126, 138, 0.15)' }, ticks: { color: '#ABAFB5' } },
                y1: { type: 'linear', display: true, position: 'right', min: 40, max: 100, title: { display: true, text: 'Accuracy %', color: '#52B788' }, grid: { drawOnChartArea: false }, ticks: { color: '#ABAFB5' } }
            }
        }
    });

    // Chart 2: Finger Keystrokes & Errors
    const fingerKeys = [
        'left_pinky', 'left_ring', 'left_middle', 'left_index',
        'right_index', 'right_middle', 'right_ring', 'right_pinky'
    ];
    
    const fingerStrokes = {};
    const fingerErrs = {};

    fingerKeys.forEach(f => {
        fingerStrokes[f] = 0;
        fingerErrs[f] = 0;
        sessions.forEach(s => {
            if (s.finger_keystrokes && s.finger_keystrokes[f]) fingerStrokes[f] += s.finger_keystrokes[f];
            if (s.finger_errors && s.finger_errors[f]) fingerErrs[f] += s.finger_errors[f];
        });
    });

    const fingerLabels = fingerKeys.map(f => FINGER_DISPLAY_NAMES[f]);
    const strokesData = fingerKeys.map(f => fingerStrokes[f]);
    const errorsData = fingerKeys.map(f => fingerErrs[f]);

    const loadCtx = document.getElementById('fingerLoadChart').getContext('2d');
    fingerLoadChartInstance = new Chart(loadCtx, {
        type: 'bar',
        data: {
            labels: fingerLabels,
            datasets: [
                { label: 'Keystrokes', data: strokesData, backgroundColor: 'rgba(98, 35, 71, 0.75)', borderRadius: 4 },
                { label: 'Errors', data: errorsData, backgroundColor: 'rgba(224, 90, 101, 0.8)', borderRadius: 4 }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            animation: { duration: 150 },
            plugins: {
                legend: { display: true, labels: { color: '#ABAFB5', font: { family: 'Inter', size: 10 } } }
            },
            scales: {
                x: { grid: { display: false }, ticks: { color: '#ABAFB5', font: { size: 9 } } },
                y: { grid: { color: 'rgba(103, 126, 138, 0.15)' }, ticks: { color: '#ABAFB5' } }
            }
        }
    });

    // Chart 3: Level Performance Comparison
    const levelStats = { Beginner: { sumWpm: 0, count: 0 }, Intermediate: { sumWpm: 0, count: 0 }, Advanced: { sumWpm: 0, count: 0 } };
    sessions.forEach(s => {
        const lvl = s.level || 'Beginner';
        if (levelStats[lvl]) {
            levelStats[lvl].sumWpm += s.wpm;
            levelStats[lvl].count++;
        }
    });

    const levelLabels = ['Beginner', 'Intermediate', 'Advanced'];
    const levelAvgs = levelLabels.map(l => levelStats[l].count > 0 ? (levelStats[l].sumWpm / levelStats[l].count).toFixed(1) : 0);

    const levelCtx = document.getElementById('levelPerfChart').getContext('2d');
    levelPerfChartInstance = new Chart(levelCtx, {
        type: 'bar',
        data: {
            labels: levelLabels,
            datasets: [{
                label: 'Avg WPM',
                data: levelAvgs,
                backgroundColor: ['rgba(82, 183, 136, 0.5)', 'rgba(98, 35, 71, 0.65)', 'rgba(224, 180, 178, 0.65)'],
                borderColor: ['#52B788', '#622347', '#E0B4B2'],
                borderWidth: 1.5,
                borderRadius: 6
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            animation: { duration: 150 },
            plugins: { legend: { display: false } },
            scales: {
                x: { grid: { display: false }, ticks: { color: '#94a3b8' } },
                y: { grid: { color: 'rgba(255, 255, 255, 0.04)' }, ticks: { color: '#94a3b8' } }
            }
        }
    });
}

// Render Keyboard Heatmap based on state.heatmapMode
function renderKeyboardHeatmap(sessions) {
    const fingerKeys = [
        'left_pinky', 'left_ring', 'left_middle', 'left_index', 'left_thumb',
        'right_thumb', 'right_index', 'right_middle', 'right_ring', 'right_pinky'
    ];

    const fingerAvgs = {};
    fingerKeys.forEach(f => {
        let sum = 0, count = 0;
        sessions.forEach(s => {
            const acc = s.finger_accuracies[f];
            if (acc !== undefined && acc !== null) { sum += acc; count++; }
        });
        fingerAvgs[f] = count > 0 ? (sum / count) : null;
    });

    // Parse mistake matrix per key across sessions
    const keyMistakesCount = {};
    const keyStrokesCount = {};

    sessions.forEach(s => {
        if (s.top_mistakes) {
            const parts = s.top_mistakes.split(';');
            parts.forEach(p => {
                const match = p.match(/^(.+)->(.+):(\d+)$/);
                if (match) {
                    const exp = match[1].toLowerCase();
                    const cnt = parseInt(match[3], 10);
                    keyMistakesCount[exp] = (keyMistakesCount[exp] || 0) + cnt;
                }
            });
        }
        if (s.finger_keystrokes) {
            Object.entries(s.finger_keystrokes).forEach(([f, cnt]) => {
                if (cnt && FINGER_MAPPING[f]) {
                    const keys = FINGER_MAPPING[f];
                    const perKeyAvg = keys.length > 0 ? Math.round(cnt / keys.length) : 0;
                    keys.forEach(k => { keyStrokesCount[k.toLowerCase()] = (keyStrokesCount[k.toLowerCase()] || 0) + perKeyAvg; });
                }
            });
        }
    });

    const keyElements = document.querySelectorAll('.key');
    keyElements.forEach(keyEl => {
        keyEl.classList.remove('perfect', 'good', 'warning', 'poor');
        
        const keyValue = keyEl.getAttribute('data-key');
        if (!keyValue) return;

        const keyLower = keyValue.toLowerCase();
        const finger = KEY_TO_FINGER[keyLower];

        if (state.heatmapMode === 'finger') {
            if (!finger) return;
            const avgAcc = fingerAvgs[finger];
            if (avgAcc === null || avgAcc === undefined) return;

            if (avgAcc === 100.0) keyEl.classList.add('perfect');
            else if (avgAcc >= 95.0) keyEl.classList.add('good');
            else if (avgAcc >= 90.0) keyEl.classList.add('warning');
            else keyEl.classList.add('poor');

            keyEl.setAttribute('title', `Key '${keyValue.toUpperCase()}' (${FINGER_DISPLAY_NAMES[finger]}): ${avgAcc.toFixed(1)}% Acc`);
        } else if (state.heatmapMode === 'key-errors') {
            const errs = keyMistakesCount[keyLower] || 0;
            if (errs === 0) keyEl.classList.add('perfect');
            else if (errs <= 2) keyEl.classList.add('good');
            else if (errs <= 5) keyEl.classList.add('warning');
            else keyEl.classList.add('poor');

            keyEl.setAttribute('title', `Key '${keyValue.toUpperCase()}': ${errs} mistyped error(s)`);
        } else if (state.heatmapMode === 'load') {
            const strokes = keyStrokesCount[keyLower] || 0;
            if (strokes === 0) keyEl.classList.add('unused');
            else if (strokes > 50) keyEl.classList.add('poor');
            else if (strokes > 25) keyEl.classList.add('warning');
            else keyEl.classList.add('good');

            keyEl.setAttribute('title', `Key '${keyValue.toUpperCase()}': ~${strokes} keystrokes typed`);
        }
    });
}

// Render History Table Rows
function renderHistoryTable(sessions) {
    historyTbody.innerHTML = '';
    const sortedDesc = [...sessions].reverse();

    sortedDesc.forEach(s => {
        const tr = document.createElement('tr');
        
        // Date
        const tdDate = document.createElement('td');
        tdDate.textContent = s.date_time;
        tr.appendChild(tdDate);
        
        // Level & Mode
        const tdMeta = document.createElement('td');
        tdMeta.innerHTML = `<span class="tag-badge">${s.level || 'Beginner'}</span> <span class="tag-badge">${s.training_mode || 'Random'}</span>`;
        tr.appendChild(tdMeta);

        // Duration
        const tdDur = document.createElement('td');
        tdDur.textContent = s.session_duration_secs ? `${s.session_duration_secs}s` : '-';
        tr.appendChild(tdDur);

        // WPM
        const tdWpm = document.createElement('td');
        tdWpm.className = 'cell-wpm';
        tdWpm.textContent = s.wpm.toFixed(1);
        tr.appendChild(tdWpm);

        // Accuracy
        const tdAcc = document.createElement('td');
        const accVal = s.overall_accuracy !== null ? s.overall_accuracy : 100.0;
        tdAcc.innerHTML = `<span class="acc-pill ${accVal >= 95 ? 'perfect' : accVal >= 90 ? 'warning' : 'poor'}">${accVal.toFixed(1)}%</span>`;
        tr.appendChild(tdAcc);

        // Keystrokes / Errors
        const tdStrokes = document.createElement('td');
        tdStrokes.textContent = s.total_keystrokes !== null ? `${s.total_keystrokes} / ${s.error_count || 0}` : '-';
        tr.appendChild(tdStrokes);

        // Left Hand
        const tdLeft = document.createElement('td');
        const leftContainer = document.createElement('div');
        leftContainer.className = 'acc-pill-container';
        ['left_pinky', 'left_ring', 'left_middle', 'left_index', 'left_thumb'].forEach(f => {
            leftContainer.appendChild(createAccuracyPill(s.finger_accuracies[f], f.split('_')[1].charAt(0).toUpperCase()));
        });
        tdLeft.appendChild(leftContainer);
        tr.appendChild(tdLeft);

        // Right Hand
        const tdRight = document.createElement('td');
        const rightContainer = document.createElement('div');
        rightContainer.className = 'acc-pill-container';
        ['right_thumb', 'right_index', 'right_middle', 'right_ring', 'right_pinky'].forEach(f => {
            rightContainer.appendChild(createAccuracyPill(s.finger_accuracies[f], f.split('_')[1].charAt(0).toUpperCase()));
        });
        tdRight.appendChild(rightContainer);
        tr.appendChild(tdRight);

        historyTbody.appendChild(tr);
    });
}

function createAccuracyPill(acc, label) {
    const span = document.createElement('span');
    span.className = 'acc-pill';
    
    if (acc === null || acc === undefined) {
        span.textContent = `${label}: -`;
        return span;
    }

    span.textContent = `${label}: ${acc.toFixed(0)}%`;
    if (acc === 100.0) span.classList.add('perfect');
    else if (acc >= 95.0) span.classList.add('good');
    else if (acc >= 90.0) span.classList.add('warning');
    else span.classList.add('poor');

    return span;
}

// Modal Inspector
function openKeyModal(keyVal) {
    const keyLower = keyVal.toLowerCase();
    const finger = KEY_TO_FINGER[keyLower];
    const profile = state.profiles[state.activeProfileId];
    if (!profile) return;

    const filtered = getFilteredSessions(profile.sessions);

    modalKeyDisplay.textContent = keyVal === ' ' ? 'SPC' : keyVal.toUpperCase();
    modalKeyTitle.textContent = `Key '${keyVal === ' ' ? 'Space' : keyVal.toUpperCase()}' Breakdown`;
    modalFingerName.textContent = finger ? FINGER_DISPLAY_NAMES[finger] : 'Unassigned Finger';

    // Calculate mistakes for this specific key
    const mistakes = {};
    let totalErrs = 0;
    let approxStrokes = 0;

    filtered.forEach(s => {
        if (s.top_mistakes) {
            const parts = s.top_mistakes.split(';');
            parts.forEach(p => {
                const match = p.match(/^(.+)->(.+):(\d+)$/);
                if (match) {
                    const exp = match[1].toLowerCase();
                    const act = match[2].toUpperCase();
                    const cnt = parseInt(match[3], 10);
                    if (exp === keyLower) {
                        mistakes[act] = (mistakes[act] || 0) + cnt;
                        totalErrs += cnt;
                    }
                }
            });
        }
        if (finger && s.finger_keystrokes && s.finger_keystrokes[finger]) {
            const keys = FINGER_MAPPING[finger] || [];
            if (keys.length > 0) approxStrokes += Math.round(s.finger_keystrokes[finger] / keys.length);
        }
    });

    modalStatKeystrokes.textContent = approxStrokes > 0 ? `~${approxStrokes}` : totalErrs;
    modalStatErrors.textContent = totalErrs;
    
    // Finger accuracy
    let fingerAcc = 100.0;
    if (finger) {
        let sum = 0, count = 0;
        filtered.forEach(s => {
            const acc = s.finger_accuracies[finger];
            if (acc !== null && acc !== undefined) { sum += acc; count++; }
        });
        if (count > 0) fingerAcc = sum / count;
    }
    modalStatAccuracy.textContent = `${fingerAcc.toFixed(1)}%`;

    // Render mistake pills
    modalMistakesList.innerHTML = '';
    const mistakeEntries = Object.entries(mistakes).sort((a, b) => b[1] - a[1]);

    if (mistakeEntries.length === 0) {
        modalMistakesList.innerHTML = '<span style="font-size:0.8rem; color:var(--text-muted); font-style:italic;">No mistakes recorded for this key! Perfect typing accuracy.</span>';
    } else {
        mistakeEntries.forEach(([actKey, cnt]) => {
            const pill = document.createElement('div');
            pill.className = 'mistake-pill';
            pill.innerHTML = `Typed <span class="char">'${actKey}'</span> <span class="count">(${cnt}x)</span>`;
            modalMistakesList.appendChild(pill);
        });
    }

    modalOverlay.classList.remove('hidden');
}

function closeModal() {
    modalOverlay.classList.add('hidden');
}

// Sidebar Profile Switcher List
function renderProfileList() {
    profileList.innerHTML = '';
    const profileIds = Object.keys(state.profiles);
    
    if (profileIds.length === 0) {
        const li = document.createElement('li');
        li.className = 'profile-empty-state';
        li.textContent = 'No profiles loaded. Upload a CSV to begin!';
        profileList.appendChild(li);
        return;
    }

    profileIds.forEach(id => {
        const profile = state.profiles[id];
        const li = document.createElement('li');
        li.className = `profile-item ${id === state.activeProfileId ? 'active' : ''}`;
        
        const infoDiv = document.createElement('div');
        infoDiv.className = 'profile-info';
        
        const nameSpan = document.createElement('span');
        nameSpan.className = 'profile-name';
        nameSpan.textContent = profile.name;
        
        const detailSpan = document.createElement('span');
        detailSpan.className = 'profile-details';
        detailSpan.textContent = `${profile.sessions.length} sessions recorded`;
        
        infoDiv.appendChild(nameSpan);
        infoDiv.appendChild(detailSpan);
        li.appendChild(infoDiv);
        
        if (id === state.activeProfileId) {
            const activeIndicator = document.createElement('div');
            activeIndicator.className = 'profile-active-indicator';
            li.appendChild(activeIndicator);
        }

        li.addEventListener('click', () => {
            if (state.activeProfileId !== id) {
                state.activeProfileId = id;
                saveStateToLocalStorage();
                renderProfileList();
                showDashboard(id);
            }
        });

        profileList.appendChild(li);
    });
}

// Local Storage helpers
function saveStateToLocalStorage() {
    localStorage.setItem('fingerTrack_state', JSON.stringify(state));
}

function loadStateFromLocalStorage() {
    try {
        const saved = localStorage.getItem('fingerTrack_state');
        if (saved) {
            state = JSON.parse(saved);
        }
    } catch (e) {
        console.error('Failed to load local storage state:', e);
    }
}
