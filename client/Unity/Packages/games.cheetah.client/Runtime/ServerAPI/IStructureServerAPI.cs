using System.Runtime.InteropServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI
{
    public interface IStructureServerAPI
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);

        byte SetListener(ushort clientId, Listener listener);
        byte Set(ushort clientId, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data);

        byte CompareAndSet(
            ushort clientId,
            in CheetahObjectId objectId,
            ushort fieldId,
            ref CheetahBuffer currentValue,
            ref CheetahBuffer newValue,
            bool hasReset,
            ref CheetahBuffer resetValue
        );
    }
}