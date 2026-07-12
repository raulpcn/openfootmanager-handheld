import { invoke } from "@tauri-apps/api/core";

import type {
  GameApi,
  SessionApi,
  TimeApi,
  MatchApi,
  StartMatchParams,
  FinishLiveMatchResponse,
} from "./types";
import type { GameStateData } from "../store/gameStore";
import type { MatchSnapshot } from "../components/match/types";

import {
  advanceTimeWithMode,
  checkBlockingActions,
  skipToMatchDay,
  advanceToNextEvent,
  advanceOneDay,
} from "../services/advanceTimeService";
import { fetchSessionState } from "../services/sessionService";

const session: SessionApi = {
  getActiveGame: () => invoke<GameStateData>("get_active_game"),
  getActiveSaveId: () => invoke<string | null>("get_active_save_id"),
  saveGame: () => invoke("save_game"),
  exitToMenu: () => invoke("exit_to_menu"),
  fetchSessionState,
};

const time: TimeApi = {
  advanceTimeWithMode,
  checkBlockingActions: () => checkBlockingActions("gameApi"),
  skipToMatchDay,
  advanceToNextEvent,
  advanceOneDay,
};

const match: MatchApi = {
  getMatchSnapshot: () => invoke<MatchSnapshot | null>("get_match_snapshot"),

  startLiveMatch: (params: StartMatchParams) =>
    invoke<MatchSnapshot>("start_live_match", {
      fixtureIndex: params.fixtureIndex,
      mode: params.mode,
      allowsExtraTime: params.allowsExtraTime,
      homeTeamId: params.homeTeamId ?? null,
      awayTeamId: params.awayTeamId ?? null,
    }),

  finishLiveMatch: () => invoke<FinishLiveMatchResponse>("finish_live_match"),
};

export const gameApi: GameApi = { session, time, match };
