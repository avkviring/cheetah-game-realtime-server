using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Type
{
    public struct FieldValue<T>
    {
        internal CheetahObjectId objectId;
        internal ushort fieldId;
        internal T value;
    }
}