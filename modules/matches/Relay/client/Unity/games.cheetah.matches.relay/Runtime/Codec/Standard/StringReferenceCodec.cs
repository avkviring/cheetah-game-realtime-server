using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Codec.Formatter;

// ReSharper disable once CheckNamespace
namespace Cheetah.Matches.Relay.Types
{
    public class StringReferenceCodec : Codec<StringReference>
    {
        public void Decode(ref CheetahBuffer buffer, ref StringReference dest)
        {
            dest.value = StringFormatter.Instance.Read(ref buffer);
        }

        public void Encode(ref StringReference source, ref CheetahBuffer buffer)
        {
            StringFormatter.Instance.Write(source.value, ref buffer);
        }
    }
}