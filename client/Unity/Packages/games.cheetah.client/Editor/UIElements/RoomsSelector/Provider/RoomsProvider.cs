using System.Collections.Generic;
using System.Threading.Tasks;

namespace Games.Cheetah.Client.Editor.UIElements.RoomsSelector.Provider
{
    public interface RoomsProvider
    {
        Task<IList<ulong>> GetRooms();

        Task Destroy();
    }
}