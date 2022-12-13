using System.Runtime.InteropServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI
{
    public interface IDoubleServerAPI
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, double value);

        byte SetListener(ushort clientId, Listener listener);
        byte Set(ushort clientId, in CheetahObjectId objectId, ushort fieldId, double value);
        byte Increment(ushort clientId, in CheetahObjectId objectId, ushort fieldId, double increment);
    }
}