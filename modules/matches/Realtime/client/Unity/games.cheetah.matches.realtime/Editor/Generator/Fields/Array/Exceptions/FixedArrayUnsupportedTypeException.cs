using System;

namespace Cheetah.Matches.Relay.Editor.Generator.Fields.Array.Exceptions
{
    public class FixedArrayUnsupportedTypeException : Exception
    {
        public FixedArrayUnsupportedTypeException(string elementTypeName) : base("Unsupported type " + elementTypeName + " for fixed array field.")
        {
        }
    }
}