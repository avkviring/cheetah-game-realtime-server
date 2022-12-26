using Games.Cheetah.Client.Codec.Formatter;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;

namespace Games.Cheetah.Client.Codec.Standard
{
    public class StringReferenceCodec : Codec<StringReference>
    {
        public void Decode(ref NetworkBuffer buffer, ref StringReference dest)
        {
            dest.value = StringFormatter.Instance.Read(ref buffer);
        }

        public void Encode(in StringReference source, ref NetworkBuffer buffer)
        {
            StringFormatter.Instance.Write(source.value, ref buffer);
        }
    }
}