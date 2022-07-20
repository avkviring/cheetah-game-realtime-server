using System.Threading.Tasks;
using Cheetah.Matches.Relay.Editor.GRPC;

namespace Cheetah.Matches.Relay.Editor.DumpViewer.Provider
{
    public interface DumpProvider
    {
        Task<DumpResponse> Dump(ulong room);

        Task Destroy();
    }
}