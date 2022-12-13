using System.Runtime.InteropServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI
{
    public interface IEventServerAPI
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);
        
        byte SetListener(ushort clientId, Listener listener);
        byte Send(ushort clientId, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);
        byte Send(ushort clientId, ushort targetUser, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);
    }
}