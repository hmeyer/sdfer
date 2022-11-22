// https://www.shadertoy.com/view/lt3BW2

#define AA 0


// https://iquilezles.org/articles/normalsSDF
vec3 calcNormal( in vec3 pos )
{
    const float ep = 0.0001;
    vec2 e = vec2(1.0,-1.0)*0.5773;
    return normalize( e.xyy*map( pos + e.xyy*ep ) + 
					  e.yyx*map( pos + e.yyx*ep ) + 
					  e.yxy*map( pos + e.yxy*ep ) + 
					  e.xxx*map( pos + e.xxx*ep ) );
}

// https://iquilezles.org/articles/rmshadows
float calcSoftshadow( in vec3 ro, in vec3 rd, float tmin, float tmax, const float k )
{
	float res = 1.0;
    float t = tmin;
    for( int i=0; i<50; i++ )
    {
		float h = map( ro + rd*t );
        res = min( res, k*h/t );
        t += clamp( h, 0.02, 0.20 );
        if( res<0.005 || t>tmax ) break;
    }
    return clamp( res, 0.0, 1.0 );
}

void mainImage( out vec4 fragColor, in vec2 fragCoord )   
{
   vec3 tot = vec3(0.0);
   float required_detail = 0.5 / max(iResolution.x, iResolution.y);
    
    #if AA>1
    for( int m=0; m<AA; m++ )
    for( int n=0; n<AA; n++ )
    {
        // pixel coordinates
        vec2 o = vec2(float(m),float(n)) / float(AA) - 0.5;
        vec2 p = (-iResolution.xy + 2.0*(fragCoord+o))/iResolution.y;
        #else    
        vec2 p = (-iResolution.xy + 2.0*fragCoord)/iResolution.y;
        #endif
 
        vec3 ro = vec3(0.0,0.0,8.0);
        vec3 rd = normalize(vec3(p-vec2(0.0,0.0),-3.5));

        float t = 0.0001;
        for( int i=0; i<64; i++ )
        {
            vec3 p = ro + t*rd;
            float h = map(p);
            if( abs(h / t) < required_detail || t>11.0 ) break;
            t += h * 0.5;
        }

        vec3 col = vec3(0.0);

        if( t<11.0 )
        {
            vec3 pos = ro + t*rd;
            vec3 nor = calcNormal(pos);
            vec3  lig = normalize(vec3(1.0,0.8,-0.2));
            float dif = clamp(dot(nor,lig),0.0,1.0);
            float sha = calcSoftshadow( pos, lig, 0.001, 1.0, 16.0 );
            float amb = 0.5 + 0.5*nor.y;
            col = vec3(0.05,0.1,0.15)*amb + 
                  vec3(1.00,0.9,0.80)*dif*sha;
        }

        col = sqrt( col );
	    tot += col;
    #if AA>1
    }
    tot /= float(AA*AA);
    #endif

	fragColor = vec4( tot, 1.0 );
}