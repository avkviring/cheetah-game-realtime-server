using Games.Cheetah.Client.Codec.Formatter;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Codec.Standard
{
    public class StringReferenceCodec : Codec<StringReference>
    {
        public void Decode(ref CheetahBuffer buffer, ref StringReference dest)
        {
            dest.value = StringFormatter.Instance.Read(ref buffer);
        }

        public void Encode(in StringReference source, ref CheetahBuffer buffer)
        {
            StringFormatter.Instance.Write(source.value, ref buffer);
        }
    }
}