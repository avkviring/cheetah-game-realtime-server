using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Tests.Runtime.Codec.Types;
using NUnit.Framework;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Field
{
    public class TestCodecField : AbstractFieldTest<TestCodecField.Structure>
    {
        [GenerateCodec]
        public struct Inner
        {
            public int value;
        }

        [GenerateCodec]
        public struct Structure
        {
            public Inner innerValue;
        }

        protected override Structure GetSource()
        {
            return new Structure
            {
                innerValue = new Inner()
                {
                    value = 100
                }
            };
        }

        protected override void CheckResult(Structure source, Structure result)
        {
            Assert.AreEqual(source.innerValue.value, result.innerValue.value);
        }
    }
}