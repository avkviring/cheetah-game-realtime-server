using System.Runtime.InteropServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI
{
    public interface IFieldServerAPI
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, FieldType fieldType);

        byte SetListener(ushort clientId, Listener listener);
        byte Delete(ushort clientId, in CheetahObjectId objectId, FieldId fieldId);
    }
}