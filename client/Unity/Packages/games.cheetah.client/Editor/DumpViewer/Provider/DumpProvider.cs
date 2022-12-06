using System.Threading.Tasks;
using Cheetah.Matches.Realtime.GRPC.Admin;

namespace Games.Cheetah.Client.Editor.DumpViewer.Provider
{
    public interface DumpProvider
    {
        Task<DumpResponse> Dump(ulong room);

        Task Destroy();
    }
}