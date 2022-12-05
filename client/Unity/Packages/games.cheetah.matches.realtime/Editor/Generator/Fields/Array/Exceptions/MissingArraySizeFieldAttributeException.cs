using System;
using Cheetah.Matches.Realtime.Codec;

namespace Cheetah.Matches.Realtime.Editor.Generator.Fields.Array.Exceptions
{
    public class MissingArraySizeFieldAttributeException : Exception
    {
        public MissingArraySizeFieldAttributeException() : base($"Missing {nameof(ArraySizeField)} attribute.")
        {
        }
    }
}