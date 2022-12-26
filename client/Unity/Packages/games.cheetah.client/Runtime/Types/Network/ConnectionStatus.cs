namespace Games.Cheetah.Client.Types.Network
{
    public enum ConnectionStatus
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