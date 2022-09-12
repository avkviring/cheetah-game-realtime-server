namespace Cheetah.Matches.Realtime.EmbeddedServer.API
{
    public interface ServerMember
    {
        uint GetId();
        byte[] GetPrivateKey();
    }
}