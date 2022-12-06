using System.Collections.Generic;
using Games.Cheetah.Client.Editor.DumpViewer.Sections.Objects.Query;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.DumpViewer.Sections.Objects.Query
{
    public class ParserTest
    {
        [Test]
        public void ShouldParseFieldId()
        {
            Assert.AreEqual(new FieldIdRule(10), Parser.Parse("field=10"));
        }

        [Test]
        public void ShouldParseTemplate()
        {
            Assert.AreEqual(new TemplateRule(20), Parser.Parse("template=20"));
        }

        [Test]
        public void ShouldParseAccessGroups()
        {
            Assert.AreEqual(new AccessGroupsRule(20), Parser.Parse("groups=20"));
        }

        [Test]
        public void ShouldParseCreatedFlag()
        {
            Assert.AreEqual(new CreatedFlagRule(true), Parser.Parse("created=true"));
            Assert.AreEqual(new CreatedFlagRule(false), Parser.Parse("created=false"));
        }

        [Test]
        public void ShouldParseOwner()
        {
            Assert.AreEqual(new UserOwnerRule(100), Parser.Parse("owner=100")); 
            Assert.AreEqual(new RoomOwnerRule(), Parser.Parse("owner=room")); 
        }

        [Test]
        public void ShouldParseId()
        {
            Assert.AreEqual(new IdRule(100), Parser.Parse("id=100"));
        }

        [Test]
        public void ShouldParseNot()
        {
            Assert.AreEqual(new NotRule(new TemplateRule(20)), Parser.Parse("template!=20"));
        }

        [Test]
        public void ShouldParseSimpleAndQuery()
        {
            var result = Parser.Parse("template=20 && field=30");

            Assert.AreEqual(
                new AndRule(new List<Rule>
                {
                    new TemplateRule(20), new FieldIdRule(30)
                })
                , result);
        }

        [Test]
        public void ShouldParseSimpleOrQuery()
        {
            var result = Parser.Parse("template=20 || field=30");

            Assert.AreEqual(
                new OrRule(new List<Rule>
                {
                    new TemplateRule(20), new FieldIdRule(30)
                })
                , result);
        }

        [Test]
        public void ShouldParseComplexQuery()
        {
            var result = Parser.Parse("template=20 && field=30 || groups=7 && field=10");

            Assert.AreEqual(
                new OrRule(new List<Rule>
                {
                    new AndRule(new List<Rule>
                    {
                        new TemplateRule(20), new FieldIdRule(30)
                    }),
                    new AndRule(new List<Rule>
                    {
                        new AccessGroupsRule(7), new FieldIdRule(10)
                    })
                }), result);
        }
        [Test]
        public void ShouldParseComplexMoreOneAnd()
        {
            var result = Parser.Parse("template=20 && field=30 && groups=40");

            Assert.AreEqual(
                    new AndRule(new List<Rule>
                    {
                        new TemplateRule(20), new FieldIdRule(30),new AccessGroupsRule(40)
                    }),
                 result);
        }
        
        [Test]
        public void ShouldParseComplexMoreOneOr()
        {
            var result = Parser.Parse("template=20 || field=30 || groups=40");

            Assert.AreEqual(
                new OrRule(new List<Rule>
                {
                    new TemplateRule(20), new FieldIdRule(30),new AccessGroupsRule(40)
                }),
                result);
        }
        [Test]
        public void ShouldParseComplexQueryWithBracket()
        {
            var result = Parser.Parse("template=20 && field=30 && (groups=7 || field=10)");
        
            Assert.AreEqual(
                    new AndRule(new List<Rule>
                    {
                        new TemplateRule(20), new FieldIdRule(30), new OrRule(new List<Rule>
                        {
                            new AccessGroupsRule(7), new FieldIdRule(10)
                        })
                    }), result);
        }
        
        [Test]
        public void ShouldParseComplexQueryWithInternalBracket()
        {
            var result = Parser.Parse("(field=30 && (template=10 || template=40)) || (groups=7 && field=10)");
        
            Assert.AreEqual(
                new OrRule(new List<Rule>
                {
                    new AndRule(new List<Rule>
                    {
                        new FieldIdRule(30), new OrRule(new List<Rule>
                        {
                            new TemplateRule(10), new TemplateRule(40)
                        })
                    }),
                    new AndRule(new List<Rule>
                    {
                        new AccessGroupsRule(7), new FieldIdRule(10)
                    })
                }), result);
        }

        
        [Test]
        public void ShouldErrorWhenFieldNotRecognized()
        {
            try
            {
                Parser.Parse("some=10");
                Assert.Fail();
            }
            catch (ParserException e)
            {
                Assert.AreEqual(e.Message, "Field 'some' not recognized");
            }
        }

        [Test]
        public void ShouldErrorWhenWrongOperation()
        {
            try
            {
                Parser.Parse("some>10");
                Assert.Fail();
            }
            catch (ParserException e)
            {
                Assert.AreEqual(e.Message, "Syntax error in query 'some>10'");
            }
        }
        
        [Test]
        public void ShouldErrorWhenWrongNumberValue()
        {
            try
            {
                Parser.Parse("field=10template");
                Assert.Fail();
            }
            catch (ParserException e)
            {
                Assert.AreEqual(e.Message, "Rule 'field' must contains number but contains '10template'");
            }
        }
        
        [Test]
        public void ShouldErrorWhenWrongBooleanValue()
        {
            try
            {
                Parser.Parse("created=10");
                Assert.Fail();
            }
            catch (ParserException e)
            {
                Assert.AreEqual(e.Message, "Rule 'created' must contains true or false but contains '10'");
            }
        }
    }
}