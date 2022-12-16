using System.Runtime.InteropServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI
{
    public interface ILongServerAPI
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, long value);

        byte SetListener(ushort clientId, Listener listener);
        byte Set(ushort clientId, in CheetahObjectId objectId, FieldId.Long fieldId, long value);
        byte Increment(ushort clientId, in CheetahObjectId objectId, FieldId.Long fieldId, long increment);

        byte CompareAndSet(ushort clientId, in CheetahObjectId objectId, FieldId.Long fieldId, long currentValue, long newValue,
            bool hasReset, long resetValue);
    }
}