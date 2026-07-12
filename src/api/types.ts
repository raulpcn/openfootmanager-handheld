import type { GameStateData } from "../store/gameStore";
import type { SessionState } from "../services/sessionService";
import type {
  BlockerData,
  AdvanceTimeWithModeResponse,
  SkipToMatchDayResponse,
  OneDayResponse,
} from "../services/advanceTimeService";
import type { MatchSnapshot, RoundSummary } from "../components/match/types";

export type {
  BlockerData,
  AdvanceTimeWithModeResponse,
  SkipToMatchDayResponse,
  OneDayResponse,
  SessionState,
  MatchSnapshot,
  RoundSummary,
};

// ── Session ─────────────────────────────────────────────────────────

export interface SessionApi {
  getActiveGame(): Promise<GameStateData>;
  getActiveSaveId(): Promise<string | null>;
  saveGame(): Promise<void>;
  exitToMenu(): Promise<void>;
  fetchSessionState(): Promise<SessionState>;
}

// ── Time Advancement ────────────────────────────────────────────────

export interface TimeApi {
  advanceTimeWithMode(mode: string): Promise<AdvanceTimeWithModeResponse>;
  checkBlockingActions(): Promise<BlockerData[]>;
  skipToMatchDay(): Promise<SkipToMatchDayResponse>;
  advanceToNextEvent(): Promise<SkipToMatchDayResponse>;
  advanceOneDay(): Promise<OneDayResponse>;
}

// ── Match ───────────────────────────────────────────────────────────

export interface StartMatchParams {
  fixtureIndex: number;
  mode: string;
  allowsExtraTime: boolean;
  homeTeamId?: string | null;
  awayTeamId?: string | null;
}

export interface FinishLiveMatchResponse {
  game: GameStateData;
  round_summary?: RoundSummary | null;
}

export interface MatchApi {
  getMatchSnapshot(): Promise<MatchSnapshot | null>;
  startLiveMatch(params: StartMatchParams): Promise<MatchSnapshot>;
  finishLiveMatch(): Promise<FinishLiveMatchResponse>;
}

// ── Combined ────────────────────────────────────────────────────────

export interface GameApi {
  session: SessionApi;
  time: TimeApi;
  match: MatchApi;
}
