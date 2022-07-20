using Cheetah.Matches.Relay.Codec.Formatter;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Codec.Standard
{
    public class CheetahObjectIdCodec : Codec<CheetahObjectId>
    {
        private static readonly VariableSizeUIntFormatter VariableSizeFormatter = VariableSizeUIntFormatter.Instance;
        private static readonly BoolFormatter BoolFormatter = BoolFormatter.Instance;

        public void Decode(ref CheetahBuffer buffer, ref CheetahObjectId dest)
        {
            dest.id = VariableSizeFormatter.Read(ref buffer);
            dest.roomOwner = BoolFormatter.Read(ref buffer);
            if (!dest.roomOwner)
            {
                dest.memberId = (ushort)VariableSizeFormatter.Read(ref buffer);
            }
        }

        public void Encode(ref CheetahObjectId source, ref CheetahBuffer buffer)
        {
            VariableSizeFormatter.Write(source.id, ref buffer);
            BoolFormatter.Write(source.roomOwner, ref buffer);
            if (!source.roomOwner)
            {
                VariableSizeFormatter.Write(source.memberId, ref buffer);
            }
        }
    }
}