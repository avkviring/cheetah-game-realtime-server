using System;
using Cheetah.Matches.Relay.Codec;

namespace Cheetah.Matches.Relay.Editor.Generator.Fields.Array.Exceptions
{
    public class MissingArraySizeFieldException : Exception
    {
        public MissingArraySizeFieldException(string field) : base($"Field {field} set in {nameof(ArraySizeField)} but not present in structure.")
        {
        }
    }
}