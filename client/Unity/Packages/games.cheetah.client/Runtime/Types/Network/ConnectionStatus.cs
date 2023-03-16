namespace Games.Cheetah.Client.Types.Network
{
    public enum ConnectionStatus
    {
        Connecting,
        Connected,
        DisconnectedByIOError,
        DisconnectedByRetransmitWhenMaxCount,
        DisconnectedByRetransmitWhenMaxFrames,
        DisconnectedByRetransmitWhenMaxWaitAck,
        DisconnectedByTimeout,
        DisconnectedByClientStopped,
        DisconnectedByRoomDeleted,
        DisconnectedByMemberDeleted,
    }
}