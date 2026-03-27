import { get, post } from './client';
import type { StationInfo, TrainInfo, SuggestResult } from '$lib/types';

export function listStations(provider: string): Promise<StationInfo[]> {
	return get(`/api/stations/${provider}`);
}

export function searchTrains(
	provider: string,
	departure: string,
	arrival: string,
	date?: string,
	time?: string
): Promise<TrainInfo[]> {
	return post('/api/search', { provider, departure, arrival, date, time });
}

export function suggestStations(
	provider: string,
	query: string,
	mode?: string
): Promise<SuggestResult> {
	const params: Record<string, string> = { q: query };
	if (mode) params.mode = mode;
	return get(`/api/stations/${provider}/suggest`, params);
}
