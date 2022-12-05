namespace Cheetah.Matches.Realtime.Types
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