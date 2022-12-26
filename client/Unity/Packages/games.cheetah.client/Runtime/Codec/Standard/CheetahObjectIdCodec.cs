using Games.Cheetah.Client.Codec.Formatter;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.Client.Types.Object;

namespace Games.Cheetah.Client.Codec.Standard
{
    public class CheetahObjectIdCodec : Codec<NetworkObjectId>
    {
        private static readonly VariableSizeUIntFormatter VariableSizeFormatter = VariableSizeUIntFormatter.Instance;
        private static readonly BoolFormatter BoolFormatter = BoolFormatter.Instance;

        public void Decode(ref NetworkBuffer buffer, ref NetworkObjectId dest)
        {
            dest.id = VariableSizeFormatter.Read(ref buffer);
            dest.IsRoomOwner = BoolFormatter.Read(ref buffer);
            if (!dest.IsRoomOwner)
            {
                dest.memberId = (ushort)VariableSizeFormatter.Read(ref buffer);
            }
        }

        public void Encode(in NetworkObjectId source, ref NetworkBuffer buffer)
        {
            VariableSizeFormatter.Write(source.id, ref buffer);
            BoolFormatter.Write(source.IsRoomOwner, ref buffer);
            if (!source.IsRoomOwner)
            {
                VariableSizeFormatter.Write(source.memberId, ref buffer);
            }
        }
    }
}