namespace Cheetah.Matches.Realtime.EmbeddedServer.API
{
    public interface ServerRoom
    {
        ServerMember CreateMember(ulong group);
        ulong GetId();
    }
}