using System.Collections.Generic;
using System.Threading.Tasks;

namespace Cheetah.Matches.Realtime.Editor.UIElements.RoomsSelector.Provider
{
    public interface RoomsProvider
    {
        Task<IList<ulong>> GetRooms();

        Task Destroy();
    }
}