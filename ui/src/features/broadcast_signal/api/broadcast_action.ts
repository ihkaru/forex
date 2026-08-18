import type { BroadcastReceipt, TraderFamilyPort } from '@shared/ports';

export async function executeSignalBroadcast(
  traderFamily: TraderFamilyPort,
  signalId: string
): Promise<BroadcastReceipt> {
  return await traderFamily.broadcastSignal(signalId);
}
