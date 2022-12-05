using System.Linq;
using Cheetah.Matches.Realtime.Codec;
using NUnit.Framework;

namespace Cheetah.Matches.Realtime.Tests.Codec.Field
{
    public class TestFormatterReferencedArrayField : AbstractFieldTest<TestFormatterReferencedArrayField.Structure>
    {
        [GenerateCodec]
        public struct Structure
        {
            public uint size;

            [ArraySizeField(nameof(size))] public string[] stringArray;

            [ArraySizeField(nameof(size))] [VariableSizeCodec]
            public int[] variableSizeArray;
        }

        protected override void CheckResult(Structure source, Structure result)
        {
            Assert.AreEqual(source.size, result.size);
            Assert.True(source.stringArray.SequenceEqual(result.stringArray));
            Assert.True(source.variableSizeArray.SequenceEqual(result.variableSizeArray));
        }

        protected override Structure GetSource()
        {
            return new Structure
            {
                size = 2,
                stringArray = new[] { "hallo", "Meine Freund", null },
                variableSizeArray = new[] { 1, 2 }
            };
        }

        protected override Structure GetResult()
        {
            return new Structure
            {
                stringArray = new string[3],
                variableSizeArray = new int[2],
            };
        }
    }
}