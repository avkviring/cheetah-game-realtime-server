namespace Games.Cheetah.Client.Types
{
    public enum CheetahClientConnectionStatus
    {
        Connecting,
        Connected,
        DisconnectedByIOError,
        DisconnectedByRetryLimit,
        DisconnectedByTimeout,
        DisconnectedByClientStopped,
        DisconnectedByRoomDeleted,
        DisconnectedByMemberDeleted,
    }
}