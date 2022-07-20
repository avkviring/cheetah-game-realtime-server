namespace Cheetah.Matches.Relay.Types
{
    public enum CheetahClientConnectionStatus
    {
        Connecting,
        Connected,
        DisconnectedByIOError,
        DisconnectedByRetryLimit,
        DisconnectedByTimeout,
        DisconnectedByCommand,
    }
}