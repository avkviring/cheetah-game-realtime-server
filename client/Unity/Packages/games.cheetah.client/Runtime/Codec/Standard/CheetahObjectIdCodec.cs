using Games.Cheetah.Client.Codec.Formatter;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.Client.Types.Object;

namespace Games.Cheetah.Client.Codec.Standard
{
    public class CheetahObjectIdCodec : Codec<NetworkObjectId>
    {
        private static readonly VariableSizeUIntFormatter VariableSizeUIntFormatter = VariableSizeUIntFormatter.Instance;
        private static readonly VariableSizeULongFormatter VariableSizeULongFormatter = VariableSizeULongFormatter.Instance;
        private static readonly BoolFormatter BoolFormatter = BoolFormatter.Instance;

        public void Decode(ref NetworkBuffer buffer, ref NetworkObjectId dest)
        {
            dest.id = VariableSizeUIntFormatter.Read(ref buffer);
            dest.IsRoomOwner = BoolFormatter.Read(ref buffer);
            if (!dest.IsRoomOwner)
            {
                dest.memberId = (ushort)VariableSizeULongFormatter.Read(ref buffer);
            }
        }

        public void Encode(in NetworkObjectId source, ref NetworkBuffer buffer)
        {
            VariableSizeUIntFormatter.Write(source.id, ref buffer);
            BoolFormatter.Write(source.IsRoomOwner, ref buffer);
            if (!source.IsRoomOwner)
            {
                VariableSizeULongFormatter.Write(source.memberId, ref buffer);
            }
        }
    }
}