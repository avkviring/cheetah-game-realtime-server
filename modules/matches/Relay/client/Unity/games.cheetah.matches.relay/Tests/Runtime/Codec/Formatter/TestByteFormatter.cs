using Cheetah.Matches.Relay.Codec.Formatter;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
{
    public class TestByteAbstractFormatter : AbstractUnmanagedFormatterTest<byte, ByteFormatter>
    {
        protected override byte[] GetValues()
        {
            return new byte[] { 0, byte.MaxValue };
        }
    }
}