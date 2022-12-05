using System.Collections.Generic;
using System.Threading.Tasks;

namespace Cheetah.Matches.Realtime.Editor.UIElements.RoomsSelector.Provider
{
    public class TestRoomsProvider : RoomsProvider
    {
#pragma warning disable 1998
        public async Task<IList<ulong>> GetRooms()
        {
            return new List<ulong>
            {
                10,
                20,
                30
            };
        }

        public async Task Destroy()
        {
        }
#pragma warning restore 1998
    }
}