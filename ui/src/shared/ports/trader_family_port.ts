export interface BroadcastReceipt {
  signalId: string;
  channelId: string;
  postId: string;
  subscribersCount: number;
  publishedAt: string;
}

export interface TraderFamilyPort {
  broadcastSignal(signalId: string): Promise<BroadcastReceipt>;
  getChannelSubscribersCount(): Promise<number>;
}
