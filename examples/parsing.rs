use yaj::*;
fn main() {
    let json = r#"
{
	"ipAddress":	"192.168.65.110",
	"portNo":	8000,
	"protocol":	"HTTP",
	"macAddress":	"58:50:ed:12:6f:ba",
	"channelID":	1,
	"dateTime":	"2021-08-25T12:52:59+02:00",
	"activePostCount":	1,
	"isDataRetransmission":	false,
	"eventType":	"faceCapture",
	"eventState":	"active",
	"eventDescription":	"faceCapture",
	"channelName":	"",
	"faceCapture":	[{
			"targetAttrs":	{
				"deviceName":	"IP CAMERA",
				"deviceChannel":	1,
				"deviceId":	"20210825125301214005teylUQbgn3ptV5PsCKoAUm2Azb1smty1Rp64sx2fFWF",
				"faceTime":	"2021-08-25T12:52:59+02:00",
				"contentID":	"backgroundImage",
				"pId":	"2021082512530121400pLFx84FLfmykv"
			},
			"faces":	[{
					"faceId":	13,
					"faceRect":	{
						"height":	0.897,
						"width":	0.535,
						"x":	0.226,
						"y":	0.094
					},
					"age":	{
						"value":	34,
						"ageGroup":	"prime"
					},
					"gender":	{
						"value":	"male"
					},
					"glass":	{
						"value":	"no"
					},
					"mask":	{
						"value":	"no"
					},
					"faceExpression":	{
						"value":	"panic"
					},
					"beard":	{
						"value":	"no"
					},
					"hat":	{
						"value":	"no"
					},
					"contentID":	"faceImage",
					"pId":	"2021082512530121400FlnH8u59iFd0W",
					"stayDuration":	3000,
					"faceScore":	78,
					"captureEndMark":	true,
					"FacePictureRect":	{
						"height":	0.368,
						"width":	0.336,
						"x":	0.345,
						"y":	0.376
					},
					"blockingState":	"noBlocking",
					"pupilDistance":	216,
					"swingAngle":	3,
					"tiltAngle":	10
				}],
			"uid":	"20210825125301214005teylUQbgn3ptV5PsCKoAUm2Azb1smty1Rp64sx2fFWF"
		}]
}
    "#;
    eprintln!("{:#?}", parse(json));
}
