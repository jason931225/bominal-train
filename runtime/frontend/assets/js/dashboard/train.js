(() => {
  const preflightNode = document.getElementById('train-preflight');
  const form = document.getElementById('train-search-form');
  const searchSubmitButton = document.getElementById('train-search-submit');
  const statusNode = document.getElementById('train-search-status');
  const depStationOpen = document.getElementById('dep-station-open');
  const arrStationOpen = document.getElementById('arr-station-open');
  const swapButton = document.getElementById('station-swap');
  const depStationDisplay = document.getElementById('dep-station-display');
  const arrStationDisplay = document.getElementById('arr-station-display');
  const dateOpen = document.getElementById('dep-date-open');
  const dateDisplay = document.getElementById('dep-date-display');
  const passengerMinus = document.getElementById('passenger-minus');
  const passengerOpen = document.getElementById('passenger-open');
  const passengerPlus = document.getElementById('passenger-plus');
  const passengerDisplay = document.getElementById('passenger-display');
  const resultsNode = document.getElementById('train-results');
  const historyNode = document.getElementById('train-search-history');
  const activeSearchIdNode = document.getElementById('active-search-id');
  const taskAutoPayButton = document.getElementById('task-auto-pay-toggle');
  const taskNotifyEmailButton = document.getElementById('task-notify-email-toggle');
  const taskRetryExpiryButton = document.getElementById('task-retry-expiry-toggle');
  const taskCreateButton = document.getElementById('train-task-create');
  const taskStatusNode = document.getElementById('train-task-status');
  const taskLiveNode = document.getElementById('train-task-live');
  const stationModal = document.getElementById('station-picker-modal');
  const stationModalClose = document.getElementById('station-picker-close');
  const stationQuery = document.getElementById('station-picker-query');
  const stationCorrection = document.getElementById('station-picker-correction');
  const stationSuggestions = document.getElementById('station-picker-suggestions');
  const stationTabFavorites = document.getElementById('station-tab-favorites');
  const stationTabRegion = document.getElementById('station-tab-region');
  const stationPickerHelp = document.getElementById('station-picker-help');
  const stationRegionsNode = document.getElementById('station-picker-regions');
  const stationListNode = document.getElementById('station-picker-list');
  const dateModal = document.getElementById('date-picker-modal');
  const dateClose = document.getElementById('date-picker-close');
  const dateInput = document.getElementById('date-picker-input');
  const dateCancel = document.getElementById('date-picker-cancel');
  const dateApply = document.getElementById('date-picker-apply');
  const passengerModal = document.getElementById('passenger-picker-modal');
  const passengerClose = document.getElementById('passenger-picker-close');
  const passengerRows = document.getElementById('passenger-picker-rows');
  const passengerCancel = document.getElementById('passenger-picker-cancel');
  const passengerApply = document.getElementById('passenger-picker-apply');
  const currentThemeMode = () => document.body?.dataset?.themeMode === 'dark' ? 'dark' : 'light';
  const syncThemedSvgzIcons = (rootNode) => {
    const mode = currentThemeMode();
    const root = rootNode && typeof rootNode.querySelectorAll === 'function' ? rootNode : document;
    const icons = root.querySelectorAll('img[data-svgz-light][data-svgz-dark]');
    icons.forEach((icon) => {
      const nextSrc = mode === 'dark' ? icon.dataset.svgzDark : icon.dataset.svgzLight;
      if (!nextSrc || icon.getAttribute('src') === nextSrc) return;
      icon.setAttribute('src', nextSrc);
    });
  };

  const now = new Date();
  let depDate = now.toISOString().slice(0, 10);
  let depTime = String(now.getHours()).padStart(2, '0') + ':00';
  let depSelection = null;
  let arrSelection = null;
  let latestSchedules = [];
  let selectedScheduleKeys = [];
  let activeTaskSnapshot = null;
  let activeTaskEvents = [];
  let activeTaskId = '';
  let taskEventSource = null;
  let historyReloadTimer = null;
  let preflightProvidersByName = new Map();
  let modalLayerCounter = 0;
  const MODAL_BASE = 70;
  let stationPickerTarget = 'dep';
  let stationTab = 'favorites';
  let stationRegionsData = null;
  let stationFavorites = [];
  let stationSearchResults = [];
  let stationQueryCounter = 0;
  let stationSuggestDebounceTimer = null;
  let activeRegionKey = 'seoul';
  let favoriteDragState = null;
  let passengerCommitted = { adult: 1, child: 0, senior: 0, disability_1_to_3: 0, disability_4_to_6: 0 };
  let passengerDraft = { ...passengerCommitted };
  let taskAutoPay = false;
  let taskNotifyEmail = true;
  let taskRetryExpiry = false;
  const TRAIN_I18N = {
    en: {
      'workspace.title': 'Train workspace',
      'workspace.subtitle': 'Korail-inspired hybrid flow with modal selectors and station catalog safety checks.',
      'preflight.loading': 'Loading provider readiness...',
      'search.title': 'Search trains',
      'search.departure': 'Departure',
      'search.arrival': 'Arrival',
      'search.departure_date': 'Departure date',
      'search.departure_time': 'Departure time',
      'search.passengers': 'Passengers',
      'search.start': 'Start search',
      'search.latest_result': 'Latest search result',
      'search.recent': 'Recent searches',
      'search.loading_history': 'Loading history...',
      'search.select_station': 'Select station',
      'search.select_date': 'Select date',
      'search.select_time': 'Select time',
      'search.swap_stations': 'Swap stations',
      'search.none': 'none',
      'station.modal_title': 'Station picker',
      'station.search_label': 'Search station',
      'station.search_placeholder': 'Search station name or initials (Seoul, ㅅㅇ)',
      'station.tab_favorites': 'Favorites',
      'station.tab_region': 'By region',
      'station.favorite_add': 'Add to favorites',
      'station.favorite_remove': 'Remove from favorites',
      'station.favorites_hint': 'Star stations from any result. Drag the handle to reorder favorites.',
      'station.drag_handle': 'Drag to reorder',
      'station.correction_prompt': 'Did you mean {query}?',
      'date.modal_title': 'Departure date',
      'date.label': 'Date',
      'time.modal_title': 'Departure time',
      'time.hour_label': 'Hour',
      'time.desktop_label': 'Desktop free time input',
      'passenger.modal_title': 'Passengers',
      'passenger.adult': 'Adult (13+)',
      'passenger.child': 'Child (6-12)',
      'passenger.senior': 'Senior (65+)',
      'passenger.disability_1_to_3': 'Disability (level 1-3)',
      'passenger.disability_4_to_6': 'Disability (level 4-6)',
      'passenger.summary.adult': '{count} Adult(s)',
      'passenger.summary.child': '{count} Child(ren)',
      'passenger.summary.senior': '{count} Senior(s)',
      'passenger.summary.disability_1_to_3': '{count} Disability 1-3',
      'passenger.summary.disability_4_to_6': '{count} Disability 4-6',
      'passenger.count.one': '{count} passenger',
      'passenger.count.other': '{count} passengers',
      'common.close': 'Close',
      'common.cancel': 'Cancel',
      'common.apply': 'Apply',
      'empty.provider_jobs': 'No provider jobs for this search.',
      'empty.results': 'No trains returned yet.',
      'empty.history': 'No searches yet.',
      'empty.favorites': 'No favorite stations yet. Star stations from search results to save them here.',
      'empty.stations': 'No stations in this view.',
      'status.general_available': 'General ✓',
      'status.general_unavailable': 'General ✕',
      'status.special_available': 'Special ✓',
      'status.special_unavailable': 'Special ✕',
      'history.providers': 'providers',
      'error.load_history': 'Failed to load search history.',
      'error.poll_search': 'Search polling failed.',
      'error.load_snapshot': 'Could not load search snapshot.',
      'error.load_preflight': 'Failed to load preflight.',
      'error.load_station_catalog': 'Could not load station catalog.',
      'error.station_lookup': 'Station lookup failed.',
      'error.favorite_save': 'Could not save favorite stations.',
      'error.date_required': 'Departure date is required.',
      'error.passenger_required': 'At least one passenger is required.',
      'error.station_required': 'Choose departure and arrival stations.',
      'error.station_distinct': 'Departure and arrival stations must differ.',
      'error.search_failed': 'Search request failed.',
      'success.search_accepted': 'Search {searchId} accepted.',
      'provider.payment': 'Payment',
      'provider.credentials': 'Credentials',
      'provider.ready': 'ready',
      'provider.error': 'error',
      'provider.missing': 'missing',
    },
    ko: {
      'workspace.title': '기차 워크스페이스',
      'workspace.subtitle': 'Korail 스타일 모달 선택과 역 카탈로그 안전 검증 흐름을 제공합니다.',
      'preflight.loading': '공급자 준비 상태를 불러오는 중...',
      'search.title': '열차 조회',
      'search.departure': '출발',
      'search.arrival': '도착',
      'search.departure_date': '출발일',
      'search.departure_time': '출발시간',
      'search.passengers': '인원',
      'search.start': '열차조회',
      'search.latest_result': '최신 조회 결과',
      'search.recent': '최근 조회',
      'search.loading_history': '조회 이력을 불러오는 중...',
      'search.select_station': '역 선택',
      'search.select_date': '날짜 선택',
      'search.select_time': '시간 선택',
      'search.swap_stations': '출발/도착 교체',
      'search.none': '없음',
      'station.modal_title': '역 선택',
      'station.search_label': '역 검색',
      'station.search_placeholder': '역 이름 또는 초성 검색 (서울, ㅅㅇ)',
      'station.tab_favorites': '즐겨찾기',
      'station.tab_region': '지역별',
      'station.favorite_add': '즐겨찾기에 추가',
      'station.favorite_remove': '즐겨찾기에서 제거',
      'station.favorites_hint': '검색 결과에서 별표로 추가하고, 핸들을 드래그해 순서를 바꿀 수 있습니다.',
      'station.drag_handle': '순서 변경',
      'station.correction_prompt': '{query} 역을 찾으셨나요?',
      'date.modal_title': '출발일 선택',
      'date.label': '날짜',
      'time.modal_title': '출발시간 선택',
      'time.hour_label': '시간',
      'time.desktop_label': '데스크톱 시간 입력',
      'passenger.modal_title': '인원 선택',
      'passenger.adult': '어른(13세 이상)',
      'passenger.child': '어린이(6~12세)',
      'passenger.senior': '경로(65세 이상)',
      'passenger.disability_1_to_3': '중증 장애인',
      'passenger.disability_4_to_6': '경증 장애인',
      'passenger.summary.adult': '어른 {count}명',
      'passenger.summary.child': '어린이 {count}명',
      'passenger.summary.senior': '경로 {count}명',
      'passenger.summary.disability_1_to_3': '중증 장애 {count}명',
      'passenger.summary.disability_4_to_6': '경증 장애 {count}명',
      'passenger.count.one': '총 {count}명',
      'passenger.count.other': '총 {count}명',
      'common.close': '닫기',
      'common.cancel': '취소',
      'common.apply': '적용',
      'empty.provider_jobs': '공급자 작업이 없습니다.',
      'empty.results': '조회된 열차가 없습니다.',
      'empty.history': '조회 이력이 없습니다.',
      'empty.favorites': '아직 즐겨찾기 역이 없습니다. 검색 결과의 별표로 추가하세요.',
      'empty.stations': '표시할 역이 없습니다.',
      'status.general_available': '일반석 가능',
      'status.general_unavailable': '일반석 불가',
      'status.special_available': '특실 가능',
      'status.special_unavailable': '특실 불가',
      'history.providers': '공급자',
      'error.load_history': '조회 이력을 불러오지 못했습니다.',
      'error.poll_search': '조회 상태 갱신에 실패했습니다.',
      'error.load_snapshot': '조회 스냅샷을 불러오지 못했습니다.',
      'error.load_preflight': '준비 상태를 불러오지 못했습니다.',
      'error.load_station_catalog': '역 목록을 불러오지 못했습니다.',
      'error.station_lookup': '역 검색에 실패했습니다.',
      'error.favorite_save': '즐겨찾기 역을 저장하지 못했습니다.',
      'error.date_required': '출발일이 필요합니다.',
      'error.passenger_required': '최소 1명의 승객이 필요합니다.',
      'error.station_required': '출발역과 도착역을 선택하세요.',
      'error.station_distinct': '출발역과 도착역은 서로 달라야 합니다.',
      'error.search_failed': '조회 요청에 실패했습니다.',
      'success.search_accepted': '조회 {searchId} 요청이 접수되었습니다.',
      'provider.payment': '결제',
      'provider.credentials': '자격 증명',
      'provider.ready': '준비됨',
      'provider.error': '오류',
      'provider.missing': '미설정',
    },
    ja: {
      'workspace.title': '列車ワークスペース',
      'workspace.subtitle': 'Korail 風のモーダル選択と駅カタログ検証フローです。',
      'preflight.loading': 'プロバイダー準備状態を読み込み中...',
      'search.title': '列車検索',
      'search.departure': '出発',
      'search.arrival': '到着',
      'search.departure_date': '出発日',
      'search.departure_time': '出発時刻',
      'search.passengers': '人数',
      'search.start': '列車検索',
      'search.latest_result': '最新検索結果',
      'search.recent': '最近の検索',
      'search.loading_history': '検索履歴を読み込み中...',
      'search.select_station': '駅を選択',
      'search.select_date': '日付を選択',
      'search.select_time': '時刻を選択',
      'search.swap_stations': '出発/到着を入れ替え',
      'search.none': 'なし',
      'station.modal_title': '駅選択',
      'station.search_label': '駅検索',
      'station.search_placeholder': '駅名または頭子音で検索 (ソウル, ㅅㅇ)',
      'station.tab_favorites': 'お気に入り',
      'station.tab_region': '地域別',
      'station.favorite_add': 'お気に入りに追加',
      'station.favorite_remove': 'お気に入りから削除',
      'station.favorites_hint': '検索結果から星で追加し、ハンドルをドラッグして順番を変更できます。',
      'station.drag_handle': '並び替え',
      'station.correction_prompt': '{query} をお探しですか？',
      'date.modal_title': '出発日',
      'date.label': '日付',
      'time.modal_title': '出発時刻',
      'time.hour_label': '時',
      'time.desktop_label': 'デスクトップ時刻入力',
      'passenger.modal_title': '人数',
      'passenger.adult': '大人 (13歳以上)',
      'passenger.child': 'こども (6-12歳)',
      'passenger.senior': 'シニア (65歳以上)',
      'passenger.disability_1_to_3': '障害 (1-3級)',
      'passenger.disability_4_to_6': '障害 (4-6級)',
      'passenger.summary.adult': '大人 {count}名',
      'passenger.summary.child': 'こども {count}名',
      'passenger.summary.senior': 'シニア {count}名',
      'passenger.summary.disability_1_to_3': '障害 1-3級 {count}名',
      'passenger.summary.disability_4_to_6': '障害 4-6級 {count}名',
      'passenger.count.one': '合計 {count}名',
      'passenger.count.other': '合計 {count}名',
      'common.close': '閉じる',
      'common.cancel': 'キャンセル',
      'common.apply': '適用',
      'empty.provider_jobs': 'この検索のプロバイダージョブはありません。',
      'empty.results': '列車結果がありません。',
      'empty.history': '検索履歴がありません。',
      'empty.favorites': 'お気に入り駅はまだありません。検索結果から星を付けて追加してください。',
      'empty.stations': 'この表示に駅はありません。',
      'status.general_available': '普通席 ✓',
      'status.general_unavailable': '普通席 ✕',
      'status.special_available': '特室 ✓',
      'status.special_unavailable': '特室 ✕',
      'history.providers': 'プロバイダー',
      'error.load_history': '検索履歴を読み込めませんでした。',
      'error.poll_search': '検索ポーリングに失敗しました。',
      'error.load_snapshot': '検索スナップショットを読み込めませんでした。',
      'error.load_preflight': '準備状態を読み込めませんでした。',
      'error.load_station_catalog': '駅カタログを読み込めませんでした。',
      'error.station_lookup': '駅検索に失敗しました。',
      'error.favorite_save': 'お気に入り駅を保存できませんでした。',
      'error.date_required': '出発日が必要です。',
      'error.passenger_required': '少なくとも1人の乗客が必要です。',
      'error.station_required': '出発駅と到着駅を選択してください。',
      'error.station_distinct': '出発駅と到着駅は異なる必要があります。',
      'error.search_failed': '検索リクエストに失敗しました。',
      'success.search_accepted': '検索 {searchId} を受け付けました。',
      'provider.payment': '支払い',
      'provider.credentials': '認証情報',
      'provider.ready': '準備完了',
      'provider.error': 'エラー',
      'provider.missing': '未設定',
    },
  };
  const resolveLocale = () => {
    const token = String(
      document.body?.dataset?.locale || document.documentElement?.lang || 'en',
    ).trim().toLowerCase();
    const primary = token.split('-')[0];
    return primary === 'ko' || primary === 'ja' || primary === 'en' ? primary : 'en';
  };
  const activeLocale = resolveLocale();
  const t = (key, vars) => {
    const table = TRAIN_I18N[activeLocale] || TRAIN_I18N.en;
    let text = table[key] || TRAIN_I18N.en[key] || key;
    if (vars && typeof vars === 'object') {
      for (const [name, value] of Object.entries(vars)) {
        text = text.replaceAll(`{${name}}`, String(value));
      }
    }
    return text;
  };
  const applyStaticTranslations = () => {
    Array.from(document.querySelectorAll('[data-i18n]')).forEach((node) => {
      const key = node.getAttribute('data-i18n');
      if (!key) return;
      node.textContent = t(key);
    });
    Array.from(document.querySelectorAll('[data-i18n-placeholder]')).forEach((node) => {
      const key = node.getAttribute('data-i18n-placeholder');
      if (!key || !('placeholder' in node)) return;
      node.placeholder = t(key);
    });
    Array.from(document.querySelectorAll('[data-i18n-aria-label]')).forEach((node) => {
      const key = node.getAttribute('data-i18n-aria-label');
      if (!key) return;
      node.setAttribute('aria-label', t(key));
    });
  };
  const passengerKinds = [
    { key: 'adult', label: t('passenger.adult') },
    { key: 'child', label: t('passenger.child') },
    { key: 'senior', label: t('passenger.senior') },
    { key: 'disability_1_to_3', label: t('passenger.disability_1_to_3') },
    { key: 'disability_4_to_6', label: t('passenger.disability_4_to_6') },
  ];

  const escapeHtml = (value) => String(value || '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');

  const requestJson = async (url, method, payload) => {
    const response = await fetch(url, {
      method: method || 'GET',
      headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
      body: payload ? JSON.stringify(payload) : undefined,
    });
    const text = await response.text();
    let body = null;
    try { body = text ? JSON.parse(text) : null; } catch (_err) {}
    return { ok: response.ok, status: response.status, body };
  };

  const apiErrorMessage = (response, fallback) => {
    const body = response && response.body && typeof response.body === 'object' ? response.body : {};
    const message = typeof body.message === 'string' && body.message.trim() ? body.message.trim() : fallback;
    const requestId = typeof body.request_id === 'string' && body.request_id.trim()
      ? ` (request_id: ${body.request_id.trim()})`
      : '';
    return `${message}${requestId}`;
  };

  const showStatus = (kind, message) => {
    if (!statusNode) return;
    statusNode.classList.remove('hidden');
    statusNode.className = kind === 'error' ? 'mt-3 error-card' : 'mt-3 summary-card';
    statusNode.textContent = message;
  };

  const clearStatus = () => {
    if (!statusNode) return;
    statusNode.textContent = '';
    statusNode.className = 'mt-3 hidden';
  };

  const modalOpen = (node) => {
    if (!node) return;
    if (node.parentElement !== document.body) document.body.appendChild(node);
    modalLayerCounter += 1;
    node.style.zIndex = String(MODAL_BASE + modalLayerCounter);
    node.classList.remove('hidden');
    node.setAttribute('aria-hidden', 'false');
  };

  const modalClose = (node) => {
    if (!node) return;
    node.classList.add('hidden');
    node.setAttribute('aria-hidden', 'true');
    node.style.removeProperty('z-index');
    if (!document.querySelector('.app-modal-backdrop:not(.hidden)')) {
      modalLayerCounter = 0;
    }
  };

  const totalPassengers = (payload) => Object.values(payload).reduce((acc, value) => acc + Number(value || 0), 0);
  const formatPassengerCount = (count) => {
    const key = count === 1 ? 'passenger.count.one' : 'passenger.count.other';
    return t(key, { count });
  };
  const formatQuickPassengerSummary = (payload) => {
    const rows = passengerKinds
      .map((kind) => ({ kind: kind.key, count: Number(payload?.[kind.key] || 0) }))
      .filter((item) => item.count > 0);
    if (rows.length === 1) {
      const only = rows[0];
      const summaryKey = `passenger.summary.${only.kind}`;
      return t(summaryKey, { count: only.count });
    }
    return formatPassengerCount(totalPassengers(payload));
  };

  const passengerPayload = () => passengerKinds
    .map((kind) => ({ kind: kind.key, count: Number(passengerCommitted[kind.key] || 0) }))
    .filter((item) => item.count > 0);

  const capitalizeEnglishStationName = (value) => String(value || '')
    .toLowerCase()
    .replace(/\b[a-z]/g, (char) => char.toUpperCase());

  const normalizeProvider = (value) => {
    const token = String(value || '').trim().toLowerCase();
    if (token === 'ktx') return 'KTX';
    if (token === 'srt') return 'SRT';
    return token.toUpperCase();
  };

  const providerBullets = (station) => {
    const labels = Array.isArray(station.supported_providers)
      ? station.supported_providers.map(normalizeProvider)
      : [];
    const unique = [...new Set(labels)];
    unique.sort((left, right) => {
      const rank = (value) => value === 'KTX' ? 0 : (value === 'SRT' ? 1 : 2);
      return rank(left) - rank(right) || left.localeCompare(right);
    });
    return unique.length ? `• ${unique.join(' • ')}` : '';
  };

  const stationLocalizedName = (station) => {
    if (activeLocale === 'ja' && station.station_name_ja_katakana) return String(station.station_name_ja_katakana);
    if (activeLocale === 'en' && station.station_name_en) return capitalizeEnglishStationName(station.station_name_en);
    return String(station.station_name_ko || '');
  };

  const stationLabel = (station) => {
    const koName = String(station.station_name_ko || '').trim();
    if (activeLocale === 'ko' || !koName) return koName;
    const localized = stationLocalizedName(station).trim();
    if (!localized || localized === koName) return koName;
    return `${localized} (${koName})`;
  };

  const cloneStation = (station) => ({
    station_code: String(station?.station_code || '').trim(),
    station_name_ko: String(station?.station_name_ko || '').trim(),
    station_name_en: station?.station_name_en ? String(station.station_name_en).trim() : '',
    station_name_ja_katakana: String(station?.station_name_ja_katakana || '').trim(),
    supported_providers: Array.isArray(station?.supported_providers)
      ? station.supported_providers.map((value) => String(value || '').trim()).filter(Boolean)
      : [],
  });

  const cloneStationList = (stations) => Array.isArray(stations)
    ? stations.map((station) => cloneStation(station)).filter((station) => station.station_code)
    : [];

  const favoriteStationCodes = () => new Set(
    stationFavorites.map((station) => String(station.station_code || '').trim()).filter(Boolean),
  );

  const isFavoriteStation = (stationCode) => favoriteStationCodes().has(String(stationCode || '').trim());

  const setStationFavorites = (stations) => {
    stationFavorites = cloneStationList(stations);
    if (stationRegionsData && typeof stationRegionsData === 'object') {
      stationRegionsData.favorites = cloneStationList(stationFavorites);
      stationRegionsData.quick = cloneStationList(stationFavorites);
    }
  };

  const stationByCode = (stationCode) => {
    const code = String(stationCode || '').trim();
    if (!code) return null;
    const sources = [
      stationFavorites,
      stationSearchResults,
      Array.isArray(stationRegionsData?.regions)
        ? stationRegionsData.regions.flatMap((region) => Array.isArray(region?.stations) ? region.stations : [])
        : [],
    ];
    for (const source of sources) {
      const match = Array.isArray(source)
        ? source.find((station) => String(station?.station_code || '').trim() === code)
        : null;
      if (match) return cloneStation(match);
    }
    return null;
  };

  const stationPickerHasQuery = () => Boolean(String(stationQuery?.value || '').trim());

  const updateStationPickerHelp = () => {
    if (!stationPickerHelp) return;
    if (stationTab === 'favorites') {
      stationPickerHelp.textContent = t('station.favorites_hint');
      return;
    }
    stationPickerHelp.textContent = '';
  };

  const searchValidationMessage = () => {
    if (!depSelection || !arrSelection) return t('error.station_required');
    if (String(depSelection.station_code || '') === String(arrSelection.station_code || '')) {
      return t('error.station_distinct');
    }
    if (!String(depDate || '').trim()) return t('error.date_required');
    if (totalPassengers(passengerCommitted) < 1) return t('error.passenger_required');
    return '';
  };

  const syncSearchSubmitState = () => {
    if (!searchSubmitButton) return;
    const valid = !searchValidationMessage();
    searchSubmitButton.setAttribute('aria-disabled', valid ? 'false' : 'true');
    searchSubmitButton.classList.toggle('opacity-60', !valid);
    searchSubmitButton.classList.toggle('cursor-not-allowed', !valid);
    searchSubmitButton.dataset.searchReady = valid ? 'true' : 'false';
  };

  const updateDisplays = () => {
    const depLabel = depSelection
      ? `${stationLabel(depSelection)} · ${depSelection.station_code}`
      : t('search.select_station');
    const arrLabel = arrSelection
      ? `${stationLabel(arrSelection)} · ${arrSelection.station_code}`
      : t('search.select_station');
    depStationDisplay.textContent = depLabel;
    arrStationDisplay.textContent = arrLabel;
    dateDisplay.textContent = depDate || t('search.select_date');
    passengerDisplay.textContent = formatQuickPassengerSummary(passengerCommitted);
    syncSearchSubmitState();
  };

  const setTaskToggleState = (button, selected) => {
    if (!button) return;
    button.classList.toggle('provider-select-card-selected', selected);
    button.classList.toggle('txt-accent', selected);
    button.setAttribute('aria-pressed', selected ? 'true' : 'false');
  };

  const syncTaskToggleStates = () => {
    setTaskToggleState(taskAutoPayButton, taskAutoPay);
    setTaskToggleState(taskNotifyEmailButton, taskNotifyEmail);
    setTaskToggleState(taskRetryExpiryButton, taskRetryExpiry);
  };

  const incrementQuickPassenger = () => {
    const total = totalPassengers(passengerCommitted);
    if (total >= 9) return false;
    passengerCommitted.adult = Number(passengerCommitted.adult || 0) + 1;
    return true;
  };

  const decrementQuickPassenger = () => {
    const total = totalPassengers(passengerCommitted);
    if (total <= 1) return false;
    const order = ['adult', 'child', 'senior', 'disability_1_to_3', 'disability_4_to_6'];
    for (const kind of order) {
      const current = Number(passengerCommitted[kind] || 0);
      if (current > 0) {
        passengerCommitted[kind] = current - 1;
        return true;
      }
    }
    return false;
  };

  const compactDate = (value) => {
    const token = String(value || '').trim();
    if (/^\d{8}$/.test(token)) {
      return `${token.slice(0, 4)}-${token.slice(4, 6)}-${token.slice(6, 8)}`;
    }
    return token;
  };

  const compactTime = (value) => {
    const token = String(value || '').trim();
    if (/^\d{6}$/.test(token)) {
      return `${token.slice(0, 2)}:${token.slice(2, 4)}`;
    }
    if (/^\d{4}$/.test(token)) {
      return `${token.slice(0, 2)}:${token.slice(2, 4)}`;
    }
    return token;
  };

  const unixMsText = (value) => {
    const num = Number(value);
    if (!Number.isFinite(num) || num <= 0) return '—';
    return new Date(num).toLocaleString();
  };

  const scheduleKey = (item) => [
    String(item?.provider || '').toLowerCase(),
    String(item?.train_code || ''),
    String(item?.train_number || ''),
    String(item?.dep_date || ''),
    String(item?.dep_time || ''),
    String(item?.arr_date || ''),
    String(item?.arr_time || ''),
  ].join('|');

  const passengerLabelByKind = (kind) => {
    if (kind === 'adult') return t('passenger.adult');
    if (kind === 'child') return t('passenger.child');
    if (kind === 'senior') return t('passenger.senior');
    if (kind === 'disability_1_to_3') return t('passenger.disability_1_to_3');
    if (kind === 'disability_4_to_6') return t('passenger.disability_4_to_6');
    return kind;
  };

  const passengersSummary = (rows) => {
    if (!Array.isArray(rows) || !rows.length) return formatPassengerCount(0);
    return rows
      .map((row) => ({
        kind: String(row?.kind || ''),
        count: Number(row?.count || 0),
      }))
      .filter((row) => row.count > 0 && row.kind)
      .map((row) => `${passengerLabelByKind(row.kind)} ${row.count}`)
      .join(', ');
  };

  const selectedSchedules = () => {
    const index = new Map(latestSchedules.map((item) => [scheduleKey(item), item]));
    return selectedScheduleKeys
      .map((key) => index.get(key))
      .filter((item) => item && typeof item === 'object');
  };

  const setTaskStatus = (kind, message) => {
    if (!taskStatusNode) return;
    taskStatusNode.classList.remove('hidden');
    taskStatusNode.className = kind === 'error' ? 'error-card' : 'summary-card';
    taskStatusNode.textContent = message;
  };

  const clearTaskStatus = () => {
    if (!taskStatusNode) return;
    taskStatusNode.textContent = '';
    taskStatusNode.className = 'hidden';
  };

  const renderTaskLive = () => {
    if (!taskLiveNode) return;
    if (!activeTaskSnapshot) {
      taskLiveNode.classList.add('hidden');
      taskLiveNode.innerHTML = '';
      return;
    }
    const task = activeTaskSnapshot;
    const events = Array.isArray(activeTaskEvents) ? activeTaskEvents.slice(-6).reverse() : [];
    const eventRows = events.length
      ? events.map((event) => `
          <div class="summary-row">
            <span class="text-xs">${escapeHtml(String(event.event_type || 'event'))}</span>
            <span class="text-xs txt-supporting">${escapeHtml(unixMsText(Date.parse(String(event.created_at || ''))))}</span>
          </div>
        `).join('')
      : `<div class="text-xs txt-supporting">No events yet.</div>`;
    taskLiveNode.classList.remove('hidden');
    taskLiveNode.innerHTML = `
      <div class="summary-row">
        <span>Task ${escapeHtml(task.task_id || '')}</span>
        <span class="badge">${escapeHtml(String(task.state_name || 'unknown'))}</span>
      </div>
      <div class="summary-row text-xs">
        <span>Last tried</span>
        <span>${escapeHtml(unixMsText(task.last_tried_unix_ms))}</span>
      </div>
      <div class="summary-row text-xs">
        <span>Next poll</span>
        <span>${escapeHtml(unixMsText(task.next_poll_unix_ms))}</span>
      </div>
      <div class="summary-row text-xs">
        <span>Pay by</span>
        <span>${escapeHtml(unixMsText(task.pay_by_unix_ms))}</span>
      </div>
      <div class="mt-2 space-y-1">${eventRows}</div>
    `;
  };

  const scheduleHistoryReload = () => {
    if (historyReloadTimer) return;
    historyReloadTimer = setTimeout(() => {
      historyReloadTimer = null;
      loadHistory().catch(() => {});
    }, 800);
  };

  const closeTaskStream = () => {
    if (!taskEventSource) return;
    taskEventSource.close();
    taskEventSource = null;
  };

  const applyTaskSyncPayload = (payload) => {
    const snapshot = payload && typeof payload === 'object' ? payload.snapshot : null;
    if (!snapshot || typeof snapshot !== 'object') return;
    activeTaskSnapshot = snapshot.task && typeof snapshot.task === 'object' ? snapshot.task : null;
    activeTaskEvents = Array.isArray(snapshot.events) ? snapshot.events : [];
    if (activeTaskSnapshot?.task_id) {
      activeTaskId = String(activeTaskSnapshot.task_id);
    }
    renderTaskLive();
  };

  const applyTaskDeltaPayload = (payload) => {
    const ops = Array.isArray(payload?.ops) ? payload.ops : [];
    for (const op of ops) {
      if (!op || typeof op !== 'object') continue;
      const opCode = String(op.op || '');
      const path = String(op.path || '');
      if (path === '/task' && opCode === 'upsert') {
        activeTaskSnapshot = op.value && typeof op.value === 'object' ? op.value : activeTaskSnapshot;
        continue;
      }
      if (path === '/events' && opCode === 'append') {
        const next = op.value && typeof op.value === 'object' ? op.value : null;
        if (next) {
          activeTaskEvents = Array.isArray(activeTaskEvents) ? [...activeTaskEvents, next] : [next];
        }
      }
    }
    renderTaskLive();
    scheduleHistoryReload();
  };

  const openTaskStream = (taskId) => {
    closeTaskStream();
    if (!taskId) return;
    taskEventSource = new EventSource(`/api/train/tasks/${encodeURIComponent(taskId)}/stream`);
    taskEventSource.addEventListener('sync', (event) => {
      let payload = null;
      try { payload = event?.data ? JSON.parse(event.data) : null; } catch (_err) {}
      applyTaskSyncPayload(payload);
    });
    taskEventSource.addEventListener('delta', (event) => {
      let payload = null;
      try { payload = event?.data ? JSON.parse(event.data) : null; } catch (_err) {}
      applyTaskDeltaPayload(payload);
    });
    taskEventSource.addEventListener('error', () => {
      setTaskStatus('error', 'Task stream disconnected. Re-open a task to resume live updates.');
    });
  };

  const renderSchedules = () => {
    if (!Array.isArray(latestSchedules) || !latestSchedules.length) {
      resultsNode.innerHTML = `<div class="empty-card">${escapeHtml(t('empty.results'))}</div>`;
      if (taskCreateButton) taskCreateButton.disabled = true;
      activeSearchIdNode.textContent = t('search.none');
      return;
    }
    const passengers = passengersSummary(passengerPayload());
    resultsNode.innerHTML = latestSchedules.map((item) => {
      const key = scheduleKey(item);
      const priority = selectedScheduleKeys.indexOf(key) + 1;
      const selected = priority > 0;
      const ticketClass = selected
        ? 'border-emerald-400/70 bg-emerald-50/70 dark:bg-emerald-900/25'
        : 'border-slate-200/80 bg-white/65';
      return `
        <button type="button" class="summary-card w-full text-left border ${ticketClass}" data-schedule-key="${escapeHtml(key)}">
          <div class="summary-row">
            <span>${escapeHtml(normalizeProvider(item.provider))} · #${escapeHtml(item.train_number || '')}</span>
            <span class="badge">${selected ? `P${priority}` : 'select'}</span>
          </div>
          <div class="summary-row">
            <span>${escapeHtml(item.dep_station_code)} ${escapeHtml(compactTime(item.dep_time))}</span>
            <span>→</span>
            <span>${escapeHtml(item.arr_station_code)} ${escapeHtml(compactTime(item.arr_time))}</span>
          </div>
          <div class="summary-row text-xs txt-supporting">
            <span>${escapeHtml(compactDate(item.dep_date))}</span>
            <span>${item.general_seat_available ? escapeHtml(t('status.general_available')) : escapeHtml(t('status.general_unavailable'))}</span>
          </div>
          <p class="mt-1 text-xs txt-supporting">${escapeHtml(passengers)}</p>
        </button>
      `;
    }).join('');
    Array.from(resultsNode.querySelectorAll('[data-schedule-key]')).forEach((node) => {
      node.addEventListener('click', () => {
        const key = node.getAttribute('data-schedule-key');
        if (!key) return;
        const index = selectedScheduleKeys.indexOf(key);
        if (index >= 0) {
          selectedScheduleKeys.splice(index, 1);
        } else {
          selectedScheduleKeys.push(key);
        }
        renderSchedules();
      });
    });
    if (taskCreateButton) taskCreateButton.disabled = selectedScheduleKeys.length < 1;
    activeSearchIdNode.textContent = `${selectedScheduleKeys.length} selected`;
  };

  const renderHistory = (tasks) => {
    if (!Array.isArray(tasks) || !tasks.length) {
      historyNode.innerHTML = `<div class="empty-card">No tasks yet.</div>`;
      return;
    }
    historyNode.innerHTML = tasks.map((item) => {
      const selected = activeTaskId && item.task_id === activeTaskId;
      return `
        <button type="button" class="summary-card w-full text-left ${selected ? 'border border-emerald-400/60' : ''}" data-task-id="${escapeHtml(item.task_id)}">
          <div class="summary-row">
            <span>${escapeHtml(item.dep_station_code)} → ${escapeHtml(item.arr_station_code)}</span>
            <span class="badge">${escapeHtml(item.state_name || 'unknown')}</span>
          </div>
          <p class="mt-1 text-xs txt-supporting">${escapeHtml(item.dep_date)} ${escapeHtml(compactTime(item.dep_time))} · ${escapeHtml(normalizeProvider(item.provider))}</p>
          <p class="mt-1 text-xs txt-supporting">Last tried: ${escapeHtml(unixMsText(item.last_tried_unix_ms))}</p>
        </button>
      `;
    }).join('');
    Array.from(historyNode.querySelectorAll('[data-task-id]')).forEach((node) => {
      node.addEventListener('click', () => {
        const taskId = node.getAttribute('data-task-id');
        if (!taskId) return;
        loadTask(taskId).catch(() => {});
      });
    });
  };

  const loadHistory = async () => {
    const response = await requestJson('/api/train/tasks?limit=12');
    if (!response.ok) {
      historyNode.innerHTML = `<div class="error-card">${escapeHtml(apiErrorMessage(response, t('error.load_history')))}</div>`;
      return;
    }
    const tasks = response.body && Array.isArray(response.body.tasks) ? response.body.tasks : [];
    renderHistory(tasks);
  };

  const loadTask = async (taskId) => {
    const response = await requestJson(`/api/train/tasks/${encodeURIComponent(taskId)}`);
    if (!response.ok) {
      setTaskStatus('error', apiErrorMessage(response, 'Could not load task snapshot.'));
      return;
    }
    activeTaskSnapshot = response.body || null;
    activeTaskEvents = [];
    activeTaskId = activeTaskSnapshot?.task_id ? String(activeTaskSnapshot.task_id) : '';
    renderTaskLive();
    await loadHistory();
    openTaskStream(activeTaskId);
  };

  const providerAuthProbeStatus = (provider) => {
    if (!provider || typeof provider !== 'object') return '';
    const value = typeof provider.auth_probe_status === 'string'
      ? provider.auth_probe_status.trim().toLowerCase()
      : '';
    return value === 'error' || value === 'success' || value === 'skipped' ? value : '';
  };

  const providerHasError = (provider) => {
    if (!provider || typeof provider !== 'object') return false;
    const probeStatus = providerAuthProbeStatus(provider);
    if (probeStatus === 'error') return true;
    return Boolean(
      (typeof provider.error === 'string' && provider.error.trim())
      || (typeof provider.debug === 'string' && provider.debug.trim())
    );
  };

  const statusIcon = (kind, ready, hasError) => {
    const title = kind === 'payment' ? t('provider.payment') : t('provider.credentials');
    const state = hasError ? t('provider.error') : (ready ? t('provider.ready') : t('provider.missing'));
    const iconPrefix = kind === 'payment' ? 'provider-status-payment' : 'provider-status-credentials';
    const lightVariant = hasError
      ? `${iconPrefix}-red.svgz`
      : ready
        ? `${iconPrefix}-green.svgz`
        : `${iconPrefix}-gray-light.svgz`;
    const darkVariant = hasError
      ? `${iconPrefix}-red.svgz`
      : ready
        ? `${iconPrefix}-green.svgz`
        : `${iconPrefix}-gray-dark.svgz`;
    const src = currentThemeMode() === 'dark' ? darkVariant : lightVariant;
    return `
      <span class="provider-status-chip" title="${title}: ${state}" aria-label="${title}: ${state}">
        <img class="status-icon" src="/assets/icons/runtime-ui/${src}" data-svgz-light="/assets/icons/runtime-ui/${lightVariant}" data-svgz-dark="/assets/icons/runtime-ui/${darkVariant}" alt="" aria-hidden="true" />
      </span>
    `;
  };

  const renderPreflight = (preflight) => {
    const providers = Array.isArray(preflight.providers) ? preflight.providers : [];
    preflightProvidersByName = new Map(
      providers
        .map((provider) => [String(provider.provider || '').toLowerCase(), provider])
        .filter((entry) => entry[0]),
    );
    const providersByName = new Map(
      providers
        .map((provider) => [String(provider.provider || '').toLowerCase(), provider])
        .filter((entry) => entry[0]),
    );
    const toProviderCard = (name) => {
      const provider = providersByName.get(name);
      const providerReady = Boolean(provider && provider.credentials_ready);
      const providerHasDebugError = Boolean(provider) && providerHasError(provider);
      return `
        <article class="train-preflight-card">
          <span class="train-preflight-label">${escapeHtml(name.toUpperCase())}</span>
          <span class="provider-status-group">
            ${statusIcon('credentials', providerReady, providerHasDebugError)}
          </span>
        </article>
      `;
    };
    const paymentProvider = providersByName.get('ktx') || (providers.length ? providers[0] : null);
    const paymentReady = Boolean(paymentProvider && paymentProvider.payment_ready);
    const paymentHasError = Boolean(paymentProvider) && providerHasError(paymentProvider);
    preflightNode.innerHTML = `
      <div class="train-preflight-grid">
        ${toProviderCard('ktx')}
        ${toProviderCard('srt')}
        <article class="train-preflight-card">
          <span class="train-preflight-label">${escapeHtml(t('provider.payment'))}</span>
          <span class="provider-status-group">
            ${statusIcon('payment', paymentReady, paymentHasError)}
          </span>
        </article>
      </div>
    `;
    syncThemedSvgzIcons(preflightNode);
  };

  const loadPreflight = async () => {
    const response = await requestJson('/api/train/preflight');
    if (!response.ok) {
      preflightNode.innerHTML = `<div class="error-card">${escapeHtml(apiErrorMessage(response, t('error.load_preflight')))}</div>`;
      return;
    }
    renderPreflight(response.body || {});
  };

  const favoriteStarIcon = (selected) => `
    <svg class="h-4 w-4" viewBox="0 0 24 24" ${selected ? 'fill="currentColor"' : 'fill="none"'} stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="m12 3.5 2.63 5.33 5.88.86-4.25 4.14 1 5.85L12 17l-5.26 2.68 1-5.85-4.25-4.14 5.88-.86L12 3.5Z"></path>
    </svg>
  `;

  const favoriteDragHandleIcon = () => `
    <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <circle cx="9" cy="7" r="1.35"></circle>
      <circle cx="15" cy="7" r="1.35"></circle>
      <circle cx="9" cy="12" r="1.35"></circle>
      <circle cx="15" cy="12" r="1.35"></circle>
      <circle cx="9" cy="17" r="1.35"></circle>
      <circle cx="15" cy="17" r="1.35"></circle>
    </svg>
  `;

  const stationRowMarkup = (station, options) => {
    const favorite = isFavoriteStation(station.station_code);
    const showDragHandle = Boolean(options?.reorderable);
    const favoriteLabel = favorite ? t('station.favorite_remove') : t('station.favorite_add');
    return `
      <div class="summary-row gap-2" ${showDragHandle ? `data-favorite-item="${escapeHtml(station.station_code)}"` : ''}>
        ${showDragHandle ? `
          <button
            type="button"
            class="btn-ghost h-10 w-10 shrink-0 rounded-full p-0 txt-supporting"
            data-favorite-drag-handle="${escapeHtml(station.station_code)}"
            aria-label="${escapeHtml(t('station.drag_handle'))}"
            title="${escapeHtml(t('station.drag_handle'))}"
          >
            ${favoriteDragHandleIcon()}
          </button>
        ` : ''}
        <button
          type="button"
          class="min-w-0 flex-1 text-left"
          data-station-pick="${escapeHtml(station.station_code)}"
        >
          <span class="block truncate">${escapeHtml(stationLabel(station))} · ${escapeHtml(station.station_code)}</span>
          <span class="mt-0.5 block text-xs txt-supporting">${escapeHtml(providerBullets(station))}</span>
        </button>
        <button
          type="button"
          class="btn-ghost h-10 w-10 shrink-0 rounded-full p-0 ${favorite ? 'txt-accent' : 'txt-supporting'}"
          data-favorite-toggle="${escapeHtml(station.station_code)}"
          aria-pressed="${favorite ? 'true' : 'false'}"
          aria-label="${escapeHtml(favoriteLabel)}"
          title="${escapeHtml(favoriteLabel)}"
        >
          ${favoriteStarIcon(favorite)}
        </button>
      </div>
    `;
  };

  const clearFavoriteDropIndicators = () => {
    Array.from(stationListNode.querySelectorAll('[data-favorite-item]')).forEach((node) => {
      node.style.removeProperty('box-shadow');
      node.style.removeProperty('opacity');
    });
  };

  const moveFavoriteStation = (stations, draggedCode, targetCode, insertAfter) => {
    const next = cloneStationList(stations);
    const fromIndex = next.findIndex((station) => station.station_code === draggedCode);
    const targetIndex = next.findIndex((station) => station.station_code === targetCode);
    if (fromIndex === -1 || targetIndex === -1 || fromIndex === targetIndex) return next;
    const [dragged] = next.splice(fromIndex, 1);
    const adjustedTarget = next.findIndex((station) => station.station_code === targetCode);
    const insertIndex = insertAfter ? adjustedTarget + 1 : adjustedTarget;
    next.splice(insertIndex, 0, dragged);
    return next;
  };

  const persistStationFavorites = async (nextStations) => {
    const previous = cloneStationList(stationFavorites);
    setStationFavorites(nextStations);
    refreshStationPickerView();
    const response = await requestJson('/api/train/stations/favorites', 'PUT', {
      station_codes: stationFavorites.map((station) => station.station_code),
    });
    if (!response.ok) {
      setStationFavorites(previous);
      refreshStationPickerView();
      showStatus('error', apiErrorMessage(response, t('error.favorite_save')));
      return false;
    }
    setStationFavorites(Array.isArray(response.body?.favorites) ? response.body.favorites : nextStations);
    refreshStationPickerView();
    return true;
  };

  const toggleFavoriteStation = async (stationCode) => {
    const code = String(stationCode || '').trim();
    if (!code) return;
    const current = cloneStationList(stationFavorites);
    const existingIndex = current.findIndex((station) => station.station_code === code);
    if (existingIndex >= 0) {
      current.splice(existingIndex, 1);
      await persistStationFavorites(current);
      return;
    }
    const station = stationByCode(code);
    if (!station) return;
    current.push(station);
    await persistStationFavorites(current);
  };

  const endFavoriteDrag = async () => {
    if (!favoriteDragState) return;
    const state = favoriteDragState;
    favoriteDragState = null;
    window.removeEventListener('pointermove', onFavoriteDragMove);
    window.removeEventListener('pointerup', onFavoriteDragEnd);
    window.removeEventListener('pointercancel', onFavoriteDragEnd);
    clearFavoriteDropIndicators();
    if (!state.targetCode || state.targetCode === state.draggedCode) {
      refreshStationPickerView();
      return;
    }
    const reordered = moveFavoriteStation(
      stationFavorites,
      state.draggedCode,
      state.targetCode,
      state.insertAfter,
    );
    const sameOrder = reordered.every(
      (station, index) => station.station_code === stationFavorites[index]?.station_code,
    );
    if (!sameOrder) {
      await persistStationFavorites(reordered);
      return;
    }
    refreshStationPickerView();
  };

  function onFavoriteDragMove(event) {
    if (!favoriteDragState) return;
    const activeRow = Array.from(stationListNode.querySelectorAll('[data-favorite-item]'))
      .find((node) => node.getAttribute('data-favorite-item') === favoriteDragState.draggedCode);
    clearFavoriteDropIndicators();
    if (activeRow) activeRow.style.opacity = '0.58';
    const targetNode = event.target instanceof Element
      ? event.target
      : document.elementFromPoint(event.clientX, event.clientY);
    const row = targetNode?.closest?.('[data-favorite-item]');
    const targetCode = row?.getAttribute?.('data-favorite-item') || '';
    if (!targetCode || targetCode === favoriteDragState.draggedCode) {
      favoriteDragState.targetCode = '';
      return;
    }
    const rect = row.getBoundingClientRect();
    const insertAfter = event.clientY > (rect.top + (rect.height / 2));
    favoriteDragState.targetCode = targetCode;
    favoriteDragState.insertAfter = insertAfter;
    row.style.boxShadow = insertAfter
      ? '0 2px 0 0 rgba(129, 140, 248, 0.95)'
      : '0 -2px 0 0 rgba(129, 140, 248, 0.95)';
  }

  function onFavoriteDragEnd() {
    endFavoriteDrag().catch(() => {});
  }

  const bindStationListEvents = () => {
    Array.from(stationListNode.querySelectorAll('[data-station-pick]')).forEach((button) => {
      button.addEventListener('click', () => {
        const stationCode = button.getAttribute('data-station-pick');
        const station = stationByCode(stationCode);
        if (!station) return;
        if (stationPickerTarget === 'dep') {
          depSelection = station;
        } else {
          arrSelection = station;
        }
        clearStatus();
        updateDisplays();
        modalClose(stationModal);
      });
    });
    Array.from(stationListNode.querySelectorAll('[data-favorite-toggle]')).forEach((button) => {
      button.addEventListener('click', (event) => {
        event.preventDefault();
        event.stopPropagation();
        toggleFavoriteStation(button.getAttribute('data-favorite-toggle')).catch(() => {});
      });
    });
    Array.from(stationListNode.querySelectorAll('[data-favorite-drag-handle]')).forEach((button) => {
      button.addEventListener('pointerdown', (event) => {
        if (event.button !== undefined && event.button !== 0) return;
        const draggedCode = button.getAttribute('data-favorite-drag-handle');
        if (!draggedCode) return;
        event.preventDefault();
        favoriteDragState = { draggedCode, targetCode: '', insertAfter: false };
        window.addEventListener('pointermove', onFavoriteDragMove);
        window.addEventListener('pointerup', onFavoriteDragEnd);
        window.addEventListener('pointercancel', onFavoriteDragEnd);
      });
    });
  };

  const renderStationList = (stations, options) => {
    const emptyKey = options?.emptyKey || 'empty.stations';
    if (!stations || !stations.length) {
      stationListNode.innerHTML = `<div class="empty-card">${escapeHtml(t(emptyKey))}</div>`;
      return;
    }
    stationListNode.innerHTML = stations
      .map((station) => stationRowMarkup(station, options))
      .join('');
    bindStationListEvents();
  };

  const renderRegionChips = () => {
    if (!stationRegionsData) return;
    const regions = (stationRegionsData.regions || []).filter((region) => region.key !== 'all');
    stationRegionsNode.innerHTML = regions.map((region) => `
      <button type="button" class="${region.key === activeRegionKey ? 'btn-primary h-9 px-3' : 'btn-ghost h-9 px-3'}" data-region-key="${escapeHtml(region.key)}">${escapeHtml(region.label)}</button>
    `).join('');
    Array.from(stationRegionsNode.querySelectorAll('[data-region-key]')).forEach((button) => {
      button.addEventListener('click', () => {
        const key = button.getAttribute('data-region-key');
        if (!key) return;
        activeRegionKey = key;
        renderRegionChips();
      });
    });
    const region = regions.find((value) => value.key === activeRegionKey) || regions[0];
    renderStationList(region ? region.stations : [], { emptyKey: 'empty.stations' });
  };

  const renderStationTab = () => {
    if (!stationRegionsData) return;
    updateStationPickerHelp();
    stationTabFavorites.className = stationTab === 'favorites' ? 'btn-primary h-9 px-3' : 'btn-ghost h-9 px-3';
    stationTabRegion.className = stationTab === 'region' ? 'btn-primary h-9 px-3' : 'btn-ghost h-9 px-3';
    if (stationTab === 'favorites') {
      stationRegionsNode.innerHTML = '';
      renderStationList(stationFavorites, { emptyKey: 'empty.favorites', reorderable: true });
      return;
    }
    renderRegionChips();
  };

  const refreshStationPickerView = () => {
    if (stationPickerHasQuery()) {
      renderStationList(stationSearchResults, { emptyKey: 'empty.stations' });
      return;
    }
    renderStationTab();
  };

  const clearStationCorrection = () => {
    if (!stationCorrection) return;
    stationCorrection.innerHTML = '';
    stationCorrection.classList.add('hidden');
  };

  const suggestLangHint = () => {
    if (activeLocale === 'ko') return 'ko';
    if (activeLocale === 'ja') return 'ja';
    if (activeLocale === 'en') return 'en';
    return 'auto';
  };

  const suggestLayoutHint = (query) => {
    const compact = String(query || '').replace(/\s+/g, '');
    if (compact && /^[a-z0-9]+$/i.test(compact)) return 'qwerty';
    return 'auto';
  };

  const buildStationSuggestUrl = (query) => {
    const params = new URLSearchParams();
    params.set('q', query);
    params.set('limit', '10');
    params.set('apply_mode', 'suggest');
    params.set('lang_hint', suggestLangHint());
    params.set('layout_hint', suggestLayoutHint(query));
    return `/api/train/stations/suggest?${params.toString()}`;
  };

  const renderStationCorrection = (body) => {
    if (!stationCorrection) return;
    const correctedQuery = String(body?.corrected_query || '').trim();
    const autocorrectApplied = Boolean(body?.autocorrect_applied);
    const currentQuery = String(stationQuery?.value || '').trim();
    if (!autocorrectApplied || !correctedQuery || correctedQuery === currentQuery) {
      clearStationCorrection();
      return;
    }
    stationCorrection.classList.remove('hidden');
    stationCorrection.innerHTML = `
      <button type="button" class="summary-row w-full text-left" data-station-use-correction="${escapeHtml(correctedQuery)}">
        <span>${escapeHtml(t('station.correction_prompt', { query: correctedQuery }))}</span>
      </button>
    `;
    const button = stationCorrection.querySelector('[data-station-use-correction]');
    if (!button) return;
    button.addEventListener('click', async () => {
      stationQuery.value = correctedQuery;
      await queryStationSuggestions(correctedQuery);
      stationQuery.focus();
    });
  };

  const loadStationRegions = async () => {
    if (stationRegionsData) return stationRegionsData;
    const response = await requestJson('/api/train/stations/regions');
    if (!response.ok) {
      showStatus('error', apiErrorMessage(response, t('error.load_station_catalog')));
      return null;
    }
    stationRegionsData = response.body || { quick: [], regions: [] };
    setStationFavorites(Array.isArray(stationRegionsData?.favorites) ? stationRegionsData.favorites : []);
    return stationRegionsData;
  };

  const queryStationSuggestions = async (query) => {
    stationQueryCounter += 1;
    const requestId = stationQueryCounter;
    const response = await requestJson(buildStationSuggestUrl(query));
    if (requestId !== stationQueryCounter) return;
    if (!response.ok) {
      clearStationCorrection();
      stationSuggestions.innerHTML = `<div class="error-card">${escapeHtml(apiErrorMessage(response, t('error.station_lookup')))}</div>`;
      return;
    }
    renderStationCorrection(response.body);
    const suggestions = Array.isArray(response.body?.suggestions) ? response.body.suggestions : [];
    const merged = new Map();
    for (const station of suggestions) {
      const key = String(station.station_code || '').trim();
      const provider = normalizeProvider(station.provider);
      if (!merged.has(key)) {
        merged.set(key, {
          station_code: key,
          station_name_ko: station.station_name_ko || '',
          station_name_en: station.station_name_en || '',
          station_name_ja_katakana: station.station_name_ja_katakana || '',
          supported_providers: provider ? [provider] : [],
        });
      } else if (provider) {
        const existing = merged.get(key);
        if (!existing.supported_providers.includes(provider)) {
          existing.supported_providers.push(provider);
        }
      }
    }
    stationSearchResults = Array.from(merged.values()).map((station) => cloneStation(station));
    renderStationList(stationSearchResults, { emptyKey: 'empty.stations' });
  };

  const openStationPicker = async (target) => {
    stationPickerTarget = target;
    stationQuery.value = '';
    stationSearchResults = [];
    clearStationCorrection();
    stationSuggestions.innerHTML = '';
    stationQueryCounter += 1;
    if (stationSuggestDebounceTimer) {
      clearTimeout(stationSuggestDebounceTimer);
      stationSuggestDebounceTimer = null;
    }
    const loaded = await loadStationRegions();
    if (!loaded) return;
    refreshStationPickerView();
    modalOpen(stationModal);
    stationQuery.focus();
  };

  const renderPassengerRows = () => {
    if (!passengerRows) return;
    passengerRows.innerHTML = passengerKinds.map((kind) => `
      <div class="summary-row">
        <span>${escapeHtml(kind.label)}</span>
        <span class="inline-flex items-center gap-2">
          <button type="button" class="btn-ghost h-9 w-9 rounded-full p-0 text-base" data-passenger-op="minus" data-passenger-kind="${escapeHtml(kind.key)}" aria-label="Decrease ${escapeHtml(kind.label)}">−</button>
          <span class="w-6 text-center">${escapeHtml(String(passengerDraft[kind.key] || 0))}</span>
          <button type="button" class="btn-ghost h-9 w-9 rounded-full p-0 text-base" data-passenger-op="plus" data-passenger-kind="${escapeHtml(kind.key)}">＋</button>
        </span>
      </div>
    `).join('');
    Array.from(passengerRows.querySelectorAll('[data-passenger-op]')).forEach((button) => {
      button.addEventListener('click', () => {
        const kind = button.getAttribute('data-passenger-kind');
        const op = button.getAttribute('data-passenger-op');
        if (!kind || !Object.hasOwn(passengerDraft, kind)) return;
        const total = totalPassengers(passengerDraft);
        const current = Number(passengerDraft[kind] || 0);
        if (op === 'plus') {
          if (total >= 9) return;
          passengerDraft[kind] = current + 1;
        } else {
          if (total <= 1 && current <= 1) return;
          passengerDraft[kind] = Math.max(0, current - 1);
        }
        renderPassengerRows();
      });
    });
  };

  depStationOpen.addEventListener('click', () => openStationPicker('dep'));
  arrStationOpen.addEventListener('click', () => openStationPicker('arr'));
  swapButton.addEventListener('click', () => {
    const prevDep = depSelection;
    depSelection = arrSelection;
    arrSelection = prevDep;
    updateDisplays();
  });

  dateOpen.addEventListener('click', () => {
    dateInput.value = depDate;
    modalOpen(dateModal);
  });

  passengerMinus.addEventListener('click', () => {
    if (!decrementQuickPassenger()) return;
    clearStatus();
    updateDisplays();
  });

  passengerOpen.addEventListener('click', () => {
    passengerDraft = { ...passengerCommitted };
    renderPassengerRows();
    modalOpen(passengerModal);
  });

  passengerPlus.addEventListener('click', () => {
    if (!incrementQuickPassenger()) return;
    clearStatus();
    updateDisplays();
  });

  stationModalClose.addEventListener('click', () => modalClose(stationModal));
  dateClose.addEventListener('click', () => modalClose(dateModal));
  passengerClose.addEventListener('click', () => modalClose(passengerModal));
  dateCancel.addEventListener('click', () => modalClose(dateModal));
  passengerCancel.addEventListener('click', () => modalClose(passengerModal));

  stationTabFavorites.addEventListener('click', () => {
    stationTab = 'favorites';
    renderStationTab();
  });
  stationTabRegion.addEventListener('click', () => {
    stationTab = 'region';
    renderStationTab();
  });

  stationQuery.addEventListener('input', async () => {
    const value = stationQuery.value.trim();
    if (!value) {
      stationSearchResults = [];
      stationQueryCounter += 1;
      clearStationCorrection();
      if (stationSuggestDebounceTimer) {
        clearTimeout(stationSuggestDebounceTimer);
        stationSuggestDebounceTimer = null;
      }
      refreshStationPickerView();
      return;
    }
    if (stationSuggestDebounceTimer) {
      clearTimeout(stationSuggestDebounceTimer);
    }
    stationSuggestDebounceTimer = setTimeout(() => {
      stationSuggestDebounceTimer = null;
      queryStationSuggestions(value).catch(() => {});
    }, 150);
  });

  dateApply.addEventListener('click', () => {
    const pickedDate = String(dateInput.value || '').trim();
    if (!pickedDate) {
      showStatus('error', t('error.date_required'));
      return;
    }
    depDate = pickedDate;
    updateDisplays();
    modalClose(dateModal);
  });

  passengerApply.addEventListener('click', () => {
    if (totalPassengers(passengerDraft) < 1) {
      showStatus('error', t('error.passenger_required'));
      return;
    }
    passengerCommitted = { ...passengerDraft };
    clearStatus();
    updateDisplays();
    modalClose(passengerModal);
  });

  [stationModal, dateModal, passengerModal].forEach((modalNode) => {
    modalNode.addEventListener('click', (event) => {
      if (event.target === modalNode) modalClose(modalNode);
    });
  });

  form.addEventListener('submit', async (event) => {
    event.preventDefault();
    clearStatus();
    clearTaskStatus();
    const validationMessage = searchValidationMessage();
    if (validationMessage) {
      showStatus('error', validationMessage);
      syncSearchSubmitState();
      return;
    }
    const depDateCompact = String(depDate || '').replaceAll('-', '');
    const depTimeCompact = `${String(depTime || '00:00').replace(':', '')}00`;
    const response = await requestJson('/api/train/providers/all/search', 'POST', {
      dep_station_code: depSelection.station_code,
      arr_station_code: arrSelection.station_code,
      dep_date: depDateCompact,
      dep_time: depTimeCompact,
      passengers: passengerPayload(),
      available_only: true,
    });
    if (!response.ok) {
      showStatus('error', apiErrorMessage(response, t('error.search_failed')));
      return;
    }
    const body = response.body || {};
    latestSchedules = Array.isArray(body.schedules) ? body.schedules : [];
    selectedScheduleKeys = [];
    activeSearchIdNode.textContent = `${latestSchedules.length} schedules`;
    renderSchedules();
    if (Array.isArray(body.errors) && body.errors.length) {
      const message = body.errors[0]?.message || 'Provider search reported an error.';
      showStatus('error', message);
      return;
    }
    showStatus('success', `Found ${latestSchedules.length} schedules. Select one or more to create a task.`);
  });

  if (taskCreateButton) {
    taskCreateButton.addEventListener('click', async () => {
      clearTaskStatus();
      if (!depSelection || !arrSelection) {
        setTaskStatus('error', t('error.station_required'));
        return;
      }
      const schedules = selectedSchedules();
      if (!schedules.length) {
        setTaskStatus('error', 'Select at least one train schedule.');
        return;
      }
      const providerSet = new Set(schedules.map((item) => String(item.provider || '').toLowerCase()));
      if (providerSet.size !== 1) {
        setTaskStatus('error', 'Selected schedules must use a single provider.');
        return;
      }
      const provider = Array.from(providerSet)[0];
      const providerRecord = preflightProvidersByName.get(provider) || null;
      const autoPay = taskAutoPay;
      const notifyEmail = taskNotifyEmail;
      const retryOnExpiry = taskRetryExpiry;
      const depDateCompact = String(depDate || '').replaceAll('-', '');
      const depTimeCompact = `${String(depTime || '00:00').replace(':', '')}00`;
      const requestBody = {
        provider,
        dep_station_code: depSelection.station_code,
        arr_station_code: arrSelection.station_code,
        dep_date: depDateCompact,
        dep_time: depTimeCompact,
        passengers: passengerPayload(),
        candidates: schedules.map((schedule, index) => ({
          priority_index: index + 1,
          schedule,
        })),
        auto_pay: autoPay,
        notify_email: notifyEmail,
        retry_on_expiry: retryOnExpiry,
      };
      const paymentMethodRef = autoPay
        ? String(providerRecord?.payment_method_ref || '').trim()
        : '';
      if (paymentMethodRef) {
        requestBody.payment_method_ref = paymentMethodRef;
      }
      const response = await requestJson('/api/train/tasks', 'POST', requestBody);
      if (!response.ok) {
        setTaskStatus('error', apiErrorMessage(response, 'Failed to create task.'));
        return;
      }
      const snapshot = response.body || null;
      activeTaskSnapshot = snapshot;
      activeTaskEvents = [];
      activeTaskId = snapshot?.task_id ? String(snapshot.task_id) : '';
      if (activeTaskId) {
        openTaskStream(activeTaskId);
      }
      renderTaskLive();
      await loadHistory();
      setTaskStatus('success', `Task ${activeTaskId || ''} created.`);
    });
  }

  taskAutoPayButton?.addEventListener('click', () => {
    taskAutoPay = !taskAutoPay;
    syncTaskToggleStates();
  });

  taskNotifyEmailButton?.addEventListener('click', () => {
    taskNotifyEmail = !taskNotifyEmail;
    syncTaskToggleStates();
  });

  taskRetryExpiryButton?.addEventListener('click', () => {
    taskRetryExpiry = !taskRetryExpiry;
    syncTaskToggleStates();
  });

  window.addEventListener('beforeunload', () => {
    closeTaskStream();
  });

  applyStaticTranslations();
  syncTaskToggleStates();
  renderPassengerRows();
  updateDisplays();
  syncThemedSvgzIcons();
  if (document.body && typeof MutationObserver === 'function') {
    const themeObserver = new MutationObserver(() => syncThemedSvgzIcons());
    themeObserver.observe(document.body, { attributes: true, attributeFilter: ['data-theme-mode'] });
  }
  loadPreflight();
  loadHistory();
})();
