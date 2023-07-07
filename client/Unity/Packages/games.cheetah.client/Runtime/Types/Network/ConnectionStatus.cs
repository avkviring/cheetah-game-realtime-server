namespace Games.Cheetah.Client.Types.Network
{
    public enum ConnectionStatus
    {
        Connecting,
        Connected,
        DisconnectedByIOError,
        RetransmitOverflow,
        DisconnectedByTimeout,
        DisconnectedByClientStopped,
        DisconnectedByRoomDeleted,
        DisconnectedByMemberDeleted,
    }
}